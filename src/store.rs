use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use std::str::FromStr;

use crate::{
    card::{Currency, DbCard, PricedCard, StoreValue},
    utils::{get_project_dir, is_empty_entry},
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct MageDeck {
    pool: SqlitePool,
}

impl MageDeck {
    pub(crate) async fn load() -> Result<Self> {
        let project_root = get_project_dir().context("getting project root")?;
        let lead = PathBuf::from("sqlite:/");
        let db_name = lead.join(project_root).join("magedeck.db");

        let connect_opts = SqliteConnectOptions::from_str(db_name.to_str().unwrap())?
            .optimize_on_close(true, None)
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs_f64(0.1))
            .connect_with(connect_opts)
            .await
            .context("creating database")?;

        Self::setup_db(&pool).await?;
        Ok(Self { pool })
    }

    pub(crate) async fn sync(&mut self, cards: Vec<DbCard>) -> Result<()> {
        println!("[*] Populating database...");
        sqlx::query("delete from cards").execute(&self.pool).await?;

        for card in cards {
            sqlx::query(
                "insert into cards(id, name, set_tag, set_name, euro, euro_foil, usd, usd_foil, usd_etched, tix, cardmarket, cardhoarder, tcgplayer) values(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            )
            .bind(card.id)
            .bind(card.name)
            .bind(card.set)
            .bind(card.set_name)
            .bind(card.euro)
            .bind(card.euro_foil)
            .bind(card.usd)
            .bind(card.usd_foil)
            .bind(card.usd_etched)
            .bind(card.tix)
            .bind(card.cardmarket)
            .bind(card.cardhoarder)
            .bind(card.tcgplayer)
            .execute(&self.pool).await?;
        }

        println!("[*] Database synced!");
        Ok(())
    }

    pub(crate) async fn get_card(
        &mut self,
        name: &str,
        currency: Currency,
    ) -> Result<Option<PricedCard>> {
        let purchase_site = match currency {
            Currency::Euro | Currency::EuroFoil => "cardmarket",
            Currency::Usd | Currency::UsdFoil | Currency::UsdEtched => "tcgplayer",
            Currency::Tix => "cardhoarder",
        };

        let record: Vec<StoreValue> = sqlx::query_as::<_, StoreValue>(&format!(
            "select name, set_tag, set_name, min({}), {} from cards where name like '%{}%'",
            currency.to_string(),
            purchase_site,
            name
        ))
        .fetch_all(&self.pool)
        .await?;

        let card = &record[0];
        if is_empty_entry(card) {
            return Ok(None);
        }
        let card = PricedCard::new(card.to_owned(), currency);
        Ok(Some(card))
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .context("running database migrations")?;

        Ok(())
    }
}
