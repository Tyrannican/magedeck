use anyhow::{Context, Result};

use crate::card::{filter_cards, Card, DbCard};

const BULK: &str = "https://api.scryfall.com/bulk-data";

pub(crate) async fn download_cards() -> Result<Vec<DbCard>> {
    // NOTE: Data is static here from the API so unwraps are safe
    println!("[*] Downloading latest data from Scryfall");
    let resp = reqwest::get(BULK)
        .await
        .context("getting bulk data source")?
        .json::<serde_json::Value>()
        .await?;

    let data = resp["data"].as_array().unwrap();

    for item in data {
        let bulk = item["type"].as_str().unwrap();
        if bulk == "oracle_cards" {
            let download = item["download_uri"].as_str().unwrap();

            let cards = reqwest::get(download)
                .await
                .context("getting bulk oracle cards")?
                .json::<Vec<Card>>()
                .await?;

            return Ok(filter_cards(cards));
        }
    }

    anyhow::bail!("no download link!")
}
