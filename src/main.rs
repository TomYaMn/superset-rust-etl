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

// Helper to convert string to ScreenerMode (reusing your existing logic)
fn parse_mode(mode_str: &str) -> ScreenerMode {
    match mode_str {
        "--gainers" => ScreenerMode::TopGainers,
        "--losers" => ScreenerMode::TopLosers,
        "--active" => ScreenerMode::MostActive,
        "--volatile" => ScreenerMode::MostVolatile,
        "--unusual-volume" => ScreenerMode::UnusualVolume,
        "--highs" => ScreenerMode::New52WeekHigh,
        "--earnings" => ScreenerMode::EarningsToday,
        _ => ScreenerMode::Sp500Tech,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::from_filename(".env.dev").ok();
    
    // 1. Load Configuration (The application.properties equivalent)
    let settings = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()?;
    let app_config: AppConfig = settings.try_deserialize()?;

    // 2. Initialize Shared Resources (Wrap in Arc to share across scheduled jobs)
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = Arc::new(db::init_db_pool(&db_url).await?);
    
    // Note: YahooClient auth might expire over time. 
    // For a daemon, you may want to re-instantiate it inside the job, 
    // but for this example, we'll share it.
    let yahoo_client = Arc::new(YahooClient::new().await?);

    // 3. Setup Scheduler
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
                println!("Starting Scheduled Job: {}", job_name);
                
                // 1. Resolve tickers and handle the Error immediately.
                let tickers = match Screener::resolve_tickers(&yahoo_client, mode).await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Failed to resolve tickers for {}: {}", job_name, e);
                        return; // The non-Send Error is dropped right here!
                    }
                }; // The match statement ends. Result is consumed.

                // 2. Now we only have `tickers` (Vec<String>), which is perfectly safe to hold across an await.
                for symbol in tickers {
                    if let Err(e) = pipeline::run_etl_for_ticker(&db_pool, &yahoo_client, &symbol).await {
                        eprintln!("  ❌ ETL failed for {}: {}", symbol, e);
                    }
                    sleep(Duration::from_millis(1500)).await;
                }
            })
        })?;

        sched.add(job).await?;
        println!("Scheduled '{}' with cron: {}", job_cfg.name, job_cfg.cron);
    }

    // 4. Start the scheduler and keep the main thread alive
    sched.start().await?;
    println!("Scheduler started. Application is running in daemon mode.");
    
    // Keep the main thread alive indefinitely
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}