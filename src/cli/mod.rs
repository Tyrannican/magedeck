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

    /// Gets the cheapest price for a card / deck with the given currency
    Price {
        #[arg(short, long, conflicts_with = "deck")]
        card: Option<String>,

        #[arg(short, long, conflicts_with = "card")]
        deck: Option<String>,

        #[arg(long, value_enum, default_value_t = Currency::Euro)]
        currency: Currency,
    },

    /// Removes the .magedeck directory
    Clean,
}
