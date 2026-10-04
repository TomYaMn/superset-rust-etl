use crate::yahoo::YahooClient;
use std::fs;

#[derive(Clone, Debug)]
pub enum ScreenerMode {
    Sp500Tech,
    TopGainers,
    TopLosers,
    MostActive,
    MostVolatile,
    UnusualVolume,
    New52WeekHigh,
    EarningsToday,
    Custom(Vec<String>),
    CustomFile(String), // Add this new mode for custom JSON files
}

pub struct Screener;

impl Screener {
    pub async fn resolve_tickers(
        client: &YahooClient,
        mode: ScreenerMode,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        match mode {
            ScreenerMode::Sp500Tech => {
                println!("Loading predefined S&P 500 Tech json file");
                let content = fs::read_to_string("sp500_tech.json")?;
                let tickers: Vec<String> = serde_json::from_str(&content)?;
                Ok(tickers)
            }
            // NEW: Custom JSON File Reader with explicit error handling
            ScreenerMode::CustomFile(filepath) => {
                println!("Loading custom JSON file: {}", filepath);
                let content = fs::read_to_string(&filepath)
                    .map_err(|e| format!("CRITICAL ERROR: Failed to read JSON file '{}' - {}", filepath, e))?;
                let tickers: Vec<String> = serde_json::from_str(&content)
                    .map_err(|e| format!("CRITICAL ERROR: Failed to parse JSON in '{}' - {}", filepath, e))?;
                Ok(tickers)
            }
            ScreenerMode::TopGainers => {
                println!("Querying Yahoo Screener for Top Gainers...");
                Ok(client.fetch_screener("day_gainers", 25).await?)
            }
            ScreenerMode::TopLosers => {
                println!("Querying Yahoo Screener for Top Losers...");
                Ok(client.fetch_screener("day_losers", 25).await?)
            }
            ScreenerMode::MostActive => {
                println!("Querying Yahoo Screener for Most Active Volume...");
                Ok(client.fetch_screener("most_actives", 25).await?)
            }
            ScreenerMode::MostVolatile => {
                println!("Querying Yahoo Screener for Most Volatile...");
                Ok(client.fetch_screener("aggressive_small_caps", 25).await?)
            }
            ScreenerMode::UnusualVolume => {
                println!("Querying Yahoo Screener for Unusual Volume...");
                Ok(client.fetch_screener("unusual_volume", 25).await?)
            }
            ScreenerMode::New52WeekHigh => {
                println!("Querying Yahoo Screener for New 52-Week Highs...");
                Ok(client.fetch_screener("52_week_gainers", 25).await?)
            }
            ScreenerMode::EarningsToday => {
                println!("Querying Earnings Today...");
                Ok(client.fetch_screener("earnings_today", 25).await?)
            }
            ScreenerMode::Custom(list) => {
                println!("Loading Custom Watchlist...");
                Ok(list)
            }
        }
    }
}