use anyhow::Result;
use clap::Parser;

pub(crate) mod card;
pub(crate) mod cli;
pub(crate) mod loader;
pub(crate) mod store;
pub(crate) mod utils;

use cli::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init().await?,
        Commands::Sync => commands::sync().await?,
        Commands::Clean => commands::clean().await?,
        Commands::Price {
            card,
            deck,
            currency,
        } => commands::price(card, deck, currency).await?,
    }

    Ok(())
}
