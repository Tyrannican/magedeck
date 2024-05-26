pub(crate) mod commands;

use crate::card::Currency;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Initialise the project
    Init,

    /// Synchronise card data with the latest info from Scryfall
    Sync,

    /// Get a card from the database
    Get { card: String },

    /// Gets the cheapest price for a card / deck with the given currency
    Price {
        /// Individual card to price
        #[arg(short, long, conflicts_with = "deck")]
        card: Option<String>,

        /// Deck file to price
        #[arg(short, long, conflicts_with = "card")]
        deck: Option<String>,

        /// Currency format to use
        #[arg(long, value_enum, default_value_t = Currency::Euro)]
        currency: Currency,

        /// Use exact card name for search
        #[arg(short, long)]
        exact_match: bool,
    },

    /// Removes the .magedeck directory
    Clean,
}
