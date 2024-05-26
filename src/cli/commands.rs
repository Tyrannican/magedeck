use anyhow::{Context, Result};
use tokio::fs;

use crate::{
    card::Currency,
    loader::download_cards,
    store::MageDeck,
    utils::{self, get_project_dir, sanitise},
};

fn is_initialised() -> Result<bool> {
    let project_dir = get_project_dir().context("getting project directory")?;
    if !project_dir.exists() {
        println!("[*] Project is not initialised!");
        println!("[*] Run `magedeck init` first and try again.");
        return Ok(false);
    }

    Ok(true)
}

pub(crate) async fn init() -> Result<()> {
    let project_dir = get_project_dir().context("getting project directory")?;
    if !project_dir.exists() {
        fs::create_dir(&project_dir)
            .await
            .with_context(|| format!("creating magedeck directory"))?;
    }

    MageDeck::load().await.context("loading db")?;

    println!("[*] Initialised MageDeck at {}", project_dir.display());
    sync().await.context("syncing database")?;

    Ok(())
}

pub(crate) async fn sync() -> Result<()> {
    if !is_initialised()? {
        return Ok(());
    }

    let mut db = MageDeck::load().await?;
    let cards = download_cards()
        .await
        .context("downloading bulk data from scryfall")?;

    db.sync(cards).await.context("syncing data with db")?;

    Ok(())
}

pub(crate) async fn clean() -> Result<()> {
    let project_dir = get_project_dir().context("getting project directory")?;
    fs::remove_dir_all(&project_dir)
        .await
        .context("removing .magedeck directory")?;

    println!("[*] Removed MageDeck ({})", project_dir.display());
    Ok(())
}

// TODO: Cleanup and split out
pub(crate) async fn price(
    card: Option<String>,
    deck: Option<String>,
    currency: Currency,
    exact_match: bool,
) -> Result<()> {
    if !is_initialised()? {
        return Ok(());
    }

    let mut db = MageDeck::load().await.context("loading db")?;
    if let Some(name) = card {
        let name = sanitise(&name);
        match db.get_cheapest_card(&name, currency, exact_match).await? {
            Some(card) => println!("[*] {card} ({})", card.purchase_site.as_ref().unwrap()),
            None => println!("[*] No entry found for '{name}'"),
        }
    } else if let Some(deck) = deck {
        let loaded_deck = utils::load_deck(&deck).await?;
        if loaded_deck.contains_power && currency != Currency::Tix {
            println!("[*] Cheapest version of deck '{deck}': You added power and expected this to be cheap...? Away and chase yersel...");
            return Ok(());
        }
        let mut total_price = 0.0;
        let mut cheapest = (String::new(), f32::MAX);
        let mut most_expensive = (String::new(), 0.0);
        for card in loaded_deck.cards.into_iter() {
            let (quantity, name) = card;
            match db.get_cheapest_card(&name, currency, exact_match).await? {
                Some(entry) => {
                    if let Some(mut price) = entry.price {
                        if price < cheapest.1 {
                            cheapest = (entry.name.as_ref().unwrap().to_string(), price);
                        }

                        if price > most_expensive.1 {
                            most_expensive = (entry.name.as_ref().unwrap().to_string(), price);
                        }
                        price *= quantity as f32;
                        total_price += price;
                        println!(
                            "[*] {quantity}x {} - {} ({}): {}",
                            entry.name.unwrap(),
                            entry.set_name.unwrap(),
                            entry.set_tag.unwrap().to_uppercase(),
                            currency.to_price(Some(price))
                        );
                    }
                }
                None => println!("[*] No entry found for '{name}'"),
            }
        }
        println!(
            "\n[*] Cheapest version of deck '{deck}': {}",
            currency.to_price(Some(total_price))
        );
        println!(
            "[*] Cheapest card: {} {}",
            cheapest.0,
            currency.to_price(Some(cheapest.1))
        );
        println!(
            "[*] Most expensive card: {} {}",
            most_expensive.0,
            currency.to_price(Some(most_expensive.1))
        );
        println!("[*] {}", currency.to_purchase_location());
    } else {
        println!("[*] Need either `--deck` or `--card` argument to be set!");
    }

    Ok(())
}

pub(crate) async fn get(card: String) -> Result<()> {
    if !is_initialised()? {
        return Ok(());
    }

    let mut db = MageDeck::load().await?;
    let card = sanitise(&card);
    let cards = db.get_cards(&card).await?;
    if cards.is_empty() {
        println!("[*] No card matching '{card}'");
    }

    for card in cards {
        println!("{card}");
    }

    Ok(())
}
