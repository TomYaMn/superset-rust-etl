#![allow(dead_code)]

use serde::Deserialize;

// --- Yahoo Screener API Models ---
#[derive(Deserialize, Debug)]
pub struct YahooScreenerResponse {
    pub finance: ScreenerFinance,
}

#[derive(Deserialize, Debug)]
pub struct ScreenerFinance {
    pub result: Vec<ScreenerResult>,
}

#[derive(Deserialize, Debug)]
pub struct ScreenerResult {
    pub quotes: Vec<ScreenerQuote>,
}

#[derive(Deserialize, Debug)]
pub struct ScreenerQuote {
    pub symbol: String,
}

// --- Yahoo Summary API Models ---
#[derive(Deserialize, Debug)]
pub struct YahooQuoteSummaryResponse {
    #[serde(rename = "quoteSummary")]
    pub quote_summary: QuoteSummaryResult,
}

#[derive(Deserialize, Debug)]
pub struct QuoteSummaryResult {
    pub result: Option<Vec<SummaryModules>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SummaryModules {
    pub default_key_statistics: Option<DefaultKeyStatistics>,
    pub financial_data: Option<FinancialData>,
    pub earnings_history: Option<EarningsHistoryWrapper>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DefaultKeyStatistics {
    pub forward_p_e: Option<F64Value>,
    pub enterprise_to_ebitda: Option<F64Value>,
    pub shares_outstanding: Option<F64Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FinancialData {
    pub gross_margins: Option<F64Value>,
    pub operating_margins: Option<F64Value>,
    pub free_cashflow: Option<F64Value>,
    pub debt_to_equity: Option<F64Value>,
    pub revenue_growth: Option<F64Value>,
    pub total_revenue: Option<F64Value>, // Add this field
}

#[derive(Deserialize, Debug)]
pub struct EarningsHistoryWrapper {
    pub history: Option<Vec<EarningsHistoryItem>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EarningsHistoryItem {
    pub max_age: Option<i64>,
    pub eps_actual: Option<F64Value>,
    pub eps_estimate: Option<F64Value>,
    pub eps_surprise_percent: Option<F64Value>,
    pub quarter: Option<QuarterValue>,
}

#[derive(Deserialize, Debug)]
pub struct QuarterValue {
    pub fmt: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct F64Value {
    pub raw: Option<f64>,
    pub fmt: Option<String>,
}

// --- Yahoo Options API Models ---
#[derive(Deserialize, Debug)]
pub struct YahooOptionsResponse {
    #[serde(rename = "optionChain")]
    pub option_chain: OptionChain,
}

#[derive(Deserialize, Debug)]
pub struct OptionChain {
    pub result: Vec<ResultItem>,
}

#[derive(Deserialize, Debug)]
pub struct ResultItem {
    pub quote: Quote,
    pub options: Vec<OptionData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub regular_market_price: f64,
}

#[derive(Deserialize, Debug)]
pub struct OptionData {
    pub calls: Vec<Contract>,
    pub puts: Vec<Contract>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    pub contract_symbol: String,
    pub strike: f64,
    pub last_price: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<i32>,
    pub open_interest: Option<i32>,
    pub implied_volatility: Option<f64>,
    pub expiration: i64,
}

// --- Internal Domain Containers ---
pub struct FundamentalsRecord {
    pub symbol: String,
    pub revenue_growth: Option<f64>,
    pub gross_margin: Option<f64>,
    pub operating_margin: Option<f64>,
    pub free_cash_flow: Option<f64>,
    pub fcf_margin: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub ev_to_ebitda: Option<f64>,
    pub shares_outstanding: Option<f64>,
}

pub struct EarningsRecord {
    pub symbol: String,
    pub market_time: String,
    pub estimated_eps: Option<f64>,
    pub actual_eps: Option<f64>,
    pub eps_surprise_pct: Option<f64>,
}

pub struct OptionGreekSnapshot {
    pub symbol: String,
    pub contract_symbol: String,
    pub expiration_date: chrono::NaiveDate,
    pub days_to_expiration: i32,
    pub strike: f64,
    pub option_type: String,
    pub bid: f64,
    pub ask: f64,
    pub last_price: f64,
    pub volume: Option<i32>,
    pub open_interest: Option<i32>,
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
}