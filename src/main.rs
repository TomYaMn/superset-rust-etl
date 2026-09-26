mod db;
mod models;
mod pipeline;
mod screener;
mod yahoo;

use dotenv::from_filename;
use screener::{Screener, ScreenerMode};
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use yahoo::YahooClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    from_filename(".env.dev").ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.dev");

    println!("Initializing TimescaleDB pool...");
    let db_pool = db::init_db_pool(&db_url).await?;
    println!("Database connection established.");

    println!("Authenticating Yahoo Client...");
    let yahoo_client = YahooClient::new().await?;
    println!("Yahoo Client initialized.");

    // Parse CLI arguments to decide execution mode
    let args: Vec<String> = env::args().collect();
    let mode = if args.len() > 1 {
        match args[1].as_str() {
            "--gainers" => ScreenerMode::TopGainers,
            "--losers" => ScreenerMode::TopLosers,
            "--active" => ScreenerMode::MostActive,
            "--volatile" => ScreenerMode::MostVolatile,
            "--unusual-volume" => ScreenerMode::UnusualVolume,
            "--highs" => ScreenerMode::New52WeekHigh,
            "--earnings" => ScreenerMode::EarningsToday,
            "--symbols" if args.len() > 2 => {
                let symbols: Vec<String> = args[2]
                    .split(',')
                    .map(|s| s.trim().to_uppercase())
                    .collect();
                ScreenerMode::Custom(symbols)
            }
            _ => {
                println!("Unknown flag. Defaulting to S&P 500 Tech.");
                ScreenerMode::Sp500Tech
            }
        }
    } else {
        ScreenerMode::Sp500Tech // Default batch run
    };

    // Resolve the tickers from the screener
    let tickers = Screener::resolve_tickers(&yahoo_client, mode).await?;
    
    if tickers.is_empty() {
        println!("No tickers found for this scanner criteria. Yahoo may have restricted this endpoint on the free tier.");
        return Ok(());
    }

    println!("\nTickers targeted for batch execution (Total: {}):", tickers.len());
    println!("{:?}\n", tickers);

    let mut success_count = 0;
    let mut failure_count = 0;

    // Batch Processing Loop with Throttling
    for (idx, symbol) in tickers.iter().enumerate() {
        println!("[Batch Process {}/{}] Ticker: {}", idx + 1, tickers.len(), symbol);

        match pipeline::run_etl_for_ticker(&db_pool, &yahoo_client, symbol).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                eprintln!("Failed ETL for {}: {}", symbol, e);
                failure_count += 1;
            }
        }

        // Rate-limiting delay (1.5 seconds) to avoid HTTP 429 blocks
        sleep(Duration::from_millis(1500)).await;
    }

    println!("\n==================================================");
    println!("Batch Processing Complete!");
    println!("  Successful Tickers: {}", success_count);
    println!("  Failed Tickers:     {}", failure_count);
    println!("==================================================");

    Ok(())
}