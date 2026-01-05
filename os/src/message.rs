use anyhow::{Context, Result};
use log::trace;
use reqwest;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct QuoteData {
    #[serde(rename = "quoteText")]
    pub text: String,
    #[serde(rename = "quoteAuthor")]
    pub author: String,
}

pub fn fetch() -> Result<String> {
    let url = "http://api.forismatic.com/api/1.0/?method=getQuote&format=json&lang=en";
    let response = reqwest::blocking::get(url).context("fails to fetch quote data")?;
    let quote: QuoteData = response.json().context("fails to parse quote data")?;
    trace!("quote data {quote:#?}",);

    let text = quote.text.trim();
    let author = if quote.author.trim().is_empty() {
        "???".to_string()
    } else {
        quote.author.trim().to_string()
    };

    Ok(format!("{text} | by {author}"))
}
