mod db;
mod models;
mod pipeline;
mod screener;
mod yahoo;

use config::{Config, File};
use serde::Deserialize;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler};
use screener::{Screener, ScreenerMode};
use yahoo::YahooClient;
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
struct AppConfig {
    jobs: Vec<JobConfig>,
}

#[derive(Debug, Deserialize)]
struct JobConfig {
    name: String,
    cron: String,
    mode: String,
}

// Helper to convert string to ScreenerMode
fn parse_mode(mode_str: &str) -> ScreenerMode {
    let mode_str = mode_str.trim();

    if mode_str.starts_with("--file") {
        let filename = mode_str.trim_start_matches("--file").trim().to_string();
        return ScreenerMode::CustomFile(filename);
    }

    if mode_str.starts_with("--custom") {
        let tickers: Vec<String> = mode_str
            .trim_start_matches("--custom")
            .split(|c| c == ',' || c == ' ')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect();

        return ScreenerMode::Custom(tickers);
    }

    match mode_str {
        "--gainers" => ScreenerMode::TopGainers,
        "--losers" => ScreenerMode::TopLosers,
        "--active" => ScreenerMode::MostActive,
        "--volatile" => ScreenerMode::MostVolatile,
        "--unusual-volume" => ScreenerMode::UnusualVolume,
        "--highs" => ScreenerMode::New52WeekHigh,
        "--earnings" => ScreenerMode::EarningsToday,
        "--sp500" => ScreenerMode::Sp500Tech,
        _ => ScreenerMode::Sp500Tech,
    }
}

// Extracted reusable job execution logic
async fn execute_job(
    job_name: &str,
    mode: ScreenerMode,
    db_pool: Arc<PgPool>,
    yahoo_client: Arc<YahooClient>,
) {
    println!("Starting Job: {}", job_name);
    
    let tickers = match Screener::resolve_tickers(&yahoo_client, mode).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to resolve tickers for {}: {}", job_name, e);
            return;
        }
    };

    for symbol in tickers {
        if let Err(e) = pipeline::run_etl_for_ticker(&db_pool, &yahoo_client, &symbol).await {
            eprintln!("  ❌ ETL failed for {}: {}", symbol, e);
        }
        sleep(Duration::from_millis(1500)).await;
    }
    
    println!("Completed Job: {}", job_name);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::from_filename(".env.dev").ok();
    
    // Initialize Shared Resources
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = Arc::new(db::init_db_pool(&db_url).await?);
    let yahoo_client = Arc::new(YahooClient::new().await?);

    // Check if we should run the scheduler or run in manual CLI mode
    let enable_scheduler = env::var("ENABLE_SCHEDULER").unwrap_or_else(|_| "false".to_string()) == "true";

    if enable_scheduler {
        println!("ENABLE_SCHEDULER=true. Starting in daemon mode...");
        
        let settings = Config::builder()
            .add_source(File::with_name("config.toml"))
            .build()?;
        let app_config: AppConfig = settings.try_deserialize()?;

        let sched = JobScheduler::new().await?;

        for job_cfg in app_config.jobs {
            let db_pool = Arc::clone(&db_pool);
            let yahoo_client = Arc::clone(&yahoo_client);
            let mode = parse_mode(&job_cfg.mode);
            let job_name = job_cfg.name.clone();
            
            let job = Job::new_async(job_cfg.cron.as_str(), move |_uuid, mut _l| {
                let db_pool = Arc::clone(&db_pool);
                let yahoo_client = Arc::clone(&yahoo_client);
                let mode = mode.clone(); 
                let job_name = job_name.clone();

                Box::pin(async move {
                    execute_job(&job_name, mode, db_pool, yahoo_client).await;
                })
            })?;

            sched.add(job).await?;
            println!("Scheduled '{}' with cron: {}", job_cfg.name, job_cfg.cron);
        }

        sched.start().await?;
        println!("Scheduler started. Application is running in daemon mode.");
        
        // Listen for termination signals (Ctrl+C / Docker Stop) to shutdown cleanly
        tokio::signal::ctrl_c().await?;
        println!("Shutdown signal received. Exiting daemon...");
    } else {
        // MANUAL RUN MODE via CLI args
        println!("ENABLE_SCHEDULER=false. Running in manual CLI mode...");
        
        // Skip executable name and join ALL arguments with spaces
        let args: Vec<String> = env::args().skip(1).collect();
        let target_mode = if args.is_empty() {
            "--sp500".to_string()
        } else {
            args.join(" ") // Joins ["--custom", "NVDA"] into "--custom NVDA"
        };

        let mode = parse_mode(&target_mode);
        let job_name = format!("Manual Run ({})", target_mode);

        execute_job(&job_name, mode, db_pool, yahoo_client).await;
    }

    Ok(())
}