use crate::models::{EarningsRecord, FundamentalsRecord, OptionGreekSnapshot};
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn init_db_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
}

pub async fn insert_layer1_fundamentals(pool: &PgPool, record: &FundamentalsRecord) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let query = r#"
        INSERT INTO l1_fundamentals_quarterly (
            report_date, symbol, fiscal_period, revenue_growth_yoy,
            gross_margin, operating_margin, free_cash_flow,
            debt_to_equity, pe_ratio, ev_to_ebitda, shares_outstanding
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;

    sqlx::query(query)
        .bind(now)
        .bind(&record.symbol)
        .bind("CURRENT")
        .bind(record.revenue_growth)
        .bind(record.gross_margin)
        .bind(record.operating_margin)
        .bind(record.free_cash_flow)
        .bind(record.debt_to_equity)
        .bind(record.pe_ratio)
        .bind(record.ev_to_ebitda)
        .bind(record.shares_outstanding)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn insert_layer2_earnings(pool: &PgPool, record: &EarningsRecord) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let query = r#"
        INSERT INTO l2_earnings_context (
            earnings_date, symbol, market_time,
            estimated_eps, actual_eps, eps_surprise_pct
        )
        VALUES ($1, $2, $3, $4, $5, $6)
    "#;

    sqlx::query(query)
        .bind(now)
        .bind(&record.symbol)
        .bind(&record.market_time)
        .bind(record.estimated_eps)
        .bind(record.actual_eps)
        .bind(record.eps_surprise_pct)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn insert_layer3_5_greeks(pool: &PgPool, snapshot: &OptionGreekSnapshot) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let query = r#"
        INSERT INTO l3_l5_options_greeks_snapshots (
            snapshot_time, symbol, contract_symbol, expiration_date, days_to_expiration,
            strike, option_type, bid_price, ask_price, last_price, volume, open_interest,
            implied_volatility, delta, gamma, theta, vega
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
    "#;

    sqlx::query(query)
        .bind(now)
        .bind(&snapshot.symbol)
        .bind(&snapshot.contract_symbol)
        .bind(snapshot.expiration_date)
        .bind(snapshot.days_to_expiration)
        .bind(snapshot.strike)
        .bind(&snapshot.option_type)
        .bind(snapshot.bid)
        .bind(snapshot.ask)
        .bind(snapshot.last_price)
        .bind(snapshot.volume)
        .bind(snapshot.open_interest)
        .bind(snapshot.implied_volatility)
        .bind(snapshot.delta)
        .bind(snapshot.gamma)
        .bind(snapshot.theta)
        .bind(snapshot.vega)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn calculate_and_insert_layer4(
    pool: &PgPool,
    symbol: &str,
    stock_price: f64,
    atm_straddle_price: f64,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let implied_move_pct = if stock_price > 0.0 { atm_straddle_price / stock_price } else { 0.0 };

    let query = r#"
        INSERT INTO l4_volatility_reality_check (
            earnings_date, symbol, pre_er_stock_price, 
            atm_straddle_price, implied_move_pct
        )
        VALUES ($1, $2, $3, $4, $5)
    "#;

    sqlx::query(query)
        .bind(now)
        .bind(symbol)
        .bind(stock_price)
        .bind(atm_straddle_price)
        .bind(implied_move_pct)
        .execute(pool)
        .await?;

    Ok(())
}