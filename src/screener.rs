use crate::yahoo::YahooClient;

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
}

pub struct Screener;

impl Screener {
    pub async fn resolve_tickers(
        client: &YahooClient,
        mode: ScreenerMode,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match mode {
            ScreenerMode::Sp500Tech => {
                println!("Loading predefined S&P 500 Tech Watchlist...");
                Ok(vec![
                    "AAPL".to_string(), "MSFT".to_string(), "NVDA".to_string(), "GOOGL".to_string(),
                    "AMZN".to_string(), "META".to_string(), "AVGO".to_string(), "AMD".to_string(),
                    "TSLA".to_string(), "INTC".to_string(), "CRM".to_string(), "ORCL".to_string(),
                ])
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