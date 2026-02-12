use anyhow::{Context, Result};
use log::trace;
use std::env;

pub fn fetch() -> Result<String> {
    let url = env::var("MESSAGE_URL").context("MESSAGE_URL env var not set")?;
    let username = env::var("MESSAGE_USERNAME").context("MESSAGE_USERNAME env var not set")?;
    let password = env::var("MESSAGE_PASSWORD").context("MESSAGE_PASSWORD env var not set")?;

    trace!("fetching message from {}", url);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .basic_auth(username, Some(password))
        .send()
        .context("failed to fetch message")?;

    if !response.status().is_success() {
        anyhow::bail!("failed to fetch message: status {}", response.status());
    }

    let text = response.text().context("failed to read response text")?;
    trace!("fetched message: {}", text);
    // replace chars that are not in the font with ?
    let text = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric()
                || c.is_ascii_whitespace()
                || "!.,-#+?;:_'*/\"()".contains(c)
            {
                c
            } else {
                '?'
            }
        })
        .collect::<String>();

    Ok(text.trim().to_string())
}
