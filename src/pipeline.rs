use crate::db;
use crate::yahoo::YahooClient;
use sqlx::PgPool;

pub async fn run_etl_for_ticker(
    db_pool: &PgPool,
    yahoo_client: &YahooClient,
    symbol: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("--------------------------------------------------");
    println!("Starting ETL Pipeline for: {}", symbol);

    // 1. Layer 1 Execution (Fundamentals)
    if let Some(record) = yahoo_client.fetch_layer1_fundamentals(symbol).await? {
        db::insert_layer1_fundamentals(db_pool, &record).await?;
        println!("  [L1] Fundamentals inserted.");
    } else {
        println!("  [L1] Skipped (No fundamental data returned).");
    }

    // 2. Layer 2 Execution (Earnings History)
    let earnings_records = yahoo_client.fetch_layer2_earnings(symbol).await?;
    for record in &earnings_records {
        db::insert_layer2_earnings(db_pool, record).await?;
    }
    println!("  [L2] Saved {} earnings records.", earnings_records.len());

    // 3. Layer 3 & 5 Execution (Options Chains & Greeks)
    let (current_stock_price, greeks_snapshots) = yahoo_client
        .fetch_layer3_5_options_greeks(symbol, 5)
        .await?;

    for snapshot in &greeks_snapshots {
        db::insert_layer3_5_greeks(db_pool, snapshot).await?;
    }
    println!("  [L3/L5] Saved {} Option Greek snapshots.", greeks_snapshots.len());

    // 4. Layer 4 Execution (Volatility Reality Check)
    if let Some(first_contract) = greeks_snapshots.first() {
        let straddle_price = first_contract.last_price * 2.0;
        db::calculate_and_insert_layer4(
            db_pool,
            symbol,
            current_stock_price,
            straddle_price,
        )
        .await?;
        println!("  [L4] Volatility check calculated.");
    }

    println!("Completed ETL for {}", symbol);
    Ok(())
}