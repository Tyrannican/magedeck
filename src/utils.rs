use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::card::{Deck, DeckEntry, StoreValue, BASIC_LANDS};

pub(crate) fn get_project_dir() -> Result<PathBuf> {
    let Some(home) = dirs::home_dir() else {
        anyhow::bail!("unable to get user home directory");
    };

    Ok(home.join(".magedeck"))
}

pub(crate) fn is_empty_entry(entry: &StoreValue) -> bool {
    let (name, set_tag, set_name, _, _) = entry;
    if name.is_none() || set_tag.is_none() || set_name.is_none() {
        return true;
    }

    return false;
}

pub(crate) async fn load_deck(deck: impl AsRef<Path>) -> Result<Deck> {
    let content = fs::read_to_string(deck)
        .await
        .context("loading deck file")?;

    let cards = content
        .split('\n')
        .into_iter()
        .filter_map(|mut line| {
            line = line.trim();
            if !is_valid_deck_entry(line) || is_basic_land(line) {
                return None;
            }

            let Some((quantity, name)) = line.split_once(' ') else {
                return None;
            };

            let quantity = quantity.parse::<i8>().unwrap_or(1);
            let name = name.replace("'", "''");
            Some((quantity, name.to_string()))
        })
        .collect::<Vec<DeckEntry>>();

    let deck = Deck::new(cards);
    Ok(deck)
}

fn is_valid_deck_entry(card: &str) -> bool {
    if card == "Deck" || card == "Sideboard" || card.len() == 0 {
        return false;
    }

    true
}

fn is_basic_land(card: &str) -> bool {
    let card = card.to_lowercase();

    for land in BASIC_LANDS {
        if card.contains(land) {
            return true;
        }
    }

    false
}
