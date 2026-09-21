mod db;
mod models;
mod yahoo;

use dotenv::from_filename;
use std::env;
use yahoo::YahooClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Environment & Database Setup
    from_filename(".env.dev").ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.dev");

    println!("Initializing TimescaleDB pool...");
    let db_pool = db::init_db_pool(&db_url).await?;
    println!("Database connection established.");

    // 2. Initialize External Services
    println!("Authenticating Yahoo Client...");
    let yahoo_client = YahooClient::new().await?;
    println!("Yahoo Client initialized.");

    let symbol = "AAPL";
    println!("\nExecuting ETL pipeline for ticker: {}", symbol);

    // 3. Layer 1 Execution (Fundamentals)
    println!("Processing Layer 1 (Fundamentals)...");
    if let Some(record) = yahoo_client.fetch_layer1_fundamentals(symbol).await? {
        db::insert_layer1_fundamentals(&db_pool, &record).await?;
        println!("Layer 1 record saved successfully.");
    }

    // 4. Layer 2 Execution (Earnings History)
    println!("Processing Layer 2 (Earnings Context)...");
    let earnings_records = yahoo_client.fetch_layer2_earnings(symbol).await?;
    for record in &earnings_records {
        db::insert_layer2_earnings(&db_pool, record).await?;
    }
    println!("Saved {} Layer 2 records.", earnings_records.len());

    // 5. Layer 3 & 5 Execution (Options Chains & Greeks Engine)
    println!("Processing Layer 3 & 5 (Options & Local Black-Scholes Greeks)...");
    
    // Unpack the stock price and the greek snapshots returned from the API
    let (current_stock_price, greeks_snapshots) = yahoo_client
        .fetch_layer3_5_options_greeks(symbol, 5)
        .await?;

    for snapshot in &greeks_snapshots {
        db::insert_layer3_5_greeks(&db_pool, snapshot).await?;
    }
    println!("Saved {} Option Greek snapshots.", greeks_snapshots.len());

    // 6. Layer 4 Execution (Volatility Reality Check)
    println!("Automating Layer 4 (Volatility Reality Check)...");
    if let Some(first_contract) = greeks_snapshots.first() {
        // Approximate straddle price for demonstration purposes (Call Last Price * 2)
        let straddle_price = first_contract.last_price * 2.0; 
        
        db::calculate_and_insert_layer4(
            &db_pool,
            symbol,
            current_stock_price,
            straddle_price,
        )
        .await?;
        println!("Layer 4 record saved successfully.");
    }

    println!("\nPipeline executed successfully across all layers!");
    Ok(())
}