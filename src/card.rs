use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

pub(crate) const BASIC_LANDS: [&str; 6] =
    ["swamp", "island", "mountain", "plains", "forest", "wastes"];

pub(crate) const POWER: [&str; 9] = [
    "black lotus",
    "mox jet",
    "mox ruby",
    "mox sapphire",
    "mox pearl",
    "mox emerald",
    "ancestral recall",
    "timetwister",
    "time walk",
];

pub(crate) type DeckEntry = (i8, String);

pub(crate) type StoreValue = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<f32>,
    Option<String>,
);

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub(crate) enum Currency {
    Euro,
    #[clap(name = "euro_foil")]
    EuroFoil,
    Usd,
    #[clap(name = "usd_foil")]
    UsdFoil,
    #[clap(name = "usd_etched")]
    UsdEtched,
    Tix,
}

#[derive(Debug, Clone)]
pub(crate) struct Deck {
    pub(crate) cards: Vec<DeckEntry>,
    pub(crate) contains_power: bool,
}

impl Deck {
    pub(crate) fn new(cards: Vec<DeckEntry>) -> Self {
        let mut contains_power = false;
        'outer: for card in cards.iter() {
            let (_, name) = card;
            for power in POWER {
                if name.to_lowercase().contains(power) {
                    contains_power = true;
                    break 'outer;
                }
            }
        }

        Self {
            cards,
            contains_power,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Card {
    pub(crate) name: String,
    pub(crate) set: String,
    pub(crate) set_name: String,
    #[serde(rename = "purchase_uris")]
    pub(crate) purchase_links: Option<HashMap<String, String>>,
    pub(crate) prices: HashMap<String, Option<String>>,
}

// TODO: Convert to actual DB type
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromRow)]
pub(crate) struct DbCard {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    #[sqlx(rename = "set_tag")]
    pub(crate) set: Option<String>,
    pub(crate) set_name: Option<String>,
    pub(crate) euro: Option<f32>,
    pub(crate) euro_foil: Option<f32>,
    pub(crate) usd: Option<f32>,
    pub(crate) usd_foil: Option<f32>,
    pub(crate) usd_etched: Option<f32>,
    pub(crate) tix: Option<f32>,
    pub(crate) cardmarket: Option<String>,
    pub(crate) cardhoarder: Option<String>,
    pub(crate) tcgplayer: Option<String>,
}

impl Card {
    pub(crate) fn to_db_entry(self) -> DbCard {
        let mut card = DbCard::default();

        card.id = Some(uuid::Uuid::new_v4().to_string());
        card.name = Some(self.name);
        card.set = Some(self.set);
        card.set_name = Some(self.set_name);
        if let Some(p_links) = self.purchase_links {
            for (key, value) in p_links.into_iter() {
                match key.as_str() {
                    "cardmarket" => card.cardmarket = Some(value),
                    "cardhoarder" => card.cardhoarder = Some(value),
                    "tcgplayer" => card.tcgplayer = Some(value),
                    _ => unreachable!(),
                }
            }
        }

        for (currency, price) in self.prices.into_iter() {
            if let Some(price) = price {
                let price = price
                    .parse::<f32>()
                    .expect("{currency} should have a valid float: {price}");

                match currency.as_str() {
                    "eur" => card.euro = Some(price),
                    "eur_foil" => card.euro_foil = Some(price),
                    "usd" => card.usd = Some(price),
                    "usd_foil" => card.usd_foil = Some(price),
                    "usd_etched" => card.usd_etched = Some(price),
                    "tix" => card.tix = Some(price),
                    _ => unreachable!(),
                }
            }
        }

        card
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PricedCard {
    pub(crate) name: Option<String>,
    pub(crate) set_tag: Option<String>,
    pub(crate) set_name: Option<String>,
    pub(crate) price: Option<f32>,
    pub(crate) currency: Currency,
    pub(crate) purchase_site: Option<String>,
}

impl PricedCard {
    pub fn new(entry: StoreValue, currency: Currency) -> Self {
        let (name, set_tag, set_name, price, purchase_site) = entry;
        Self {
            name,
            set_tag,
            set_name,
            price,
            currency,
            purchase_site,
        }
    }
}

impl std::fmt::Display for PricedCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({}): {}",
            self.name.as_ref().unwrap(),
            self.set_name.as_ref().unwrap(),
            self.set_tag.as_ref().unwrap().to_uppercase(),
            self.currency.to_price(self.price),
        )
    }
}

impl Currency {
    pub(crate) fn to_price(&self, price: Option<f32>) -> String {
        let price = if let Some(price) = price {
            format!("{price:.2}")
        } else {
            "N/A".to_string()
        };

        match *self {
            Self::Euro => format!("{price}€"),
            Self::EuroFoil => format!("{price}€ Foil"),
            Self::Usd => format!("${price}"),
            Self::UsdFoil => format!("${price} Foil"),
            Self::UsdEtched => format!("${price} Etched"),
            Self::Tix => format!("{price} Tix"),
        }
    }

    pub(crate) fn to_string(&self) -> String {
        match *self {
            Self::Euro => "euro".to_string(),
            Self::EuroFoil => "euro_foil".to_string(),
            Self::Usd => "usd".to_string(),
            Self::UsdFoil => "usd_foil".to_string(),
            Self::UsdEtched => "usd_etched".to_string(),
            Self::Tix => "tix".to_string(),
        }
    }

    pub(crate) fn to_purchase_location(&self) -> String {
        match *self {
            Self::Euro | Self::EuroFoil => "Check https://www.cardmarket.com/en/Magic for buying options".to_string(),
            Self::Usd| Self::UsdFoil | Self::UsdEtched => "Check https://www.tcgplayer.com/search/magic/product?productLineName=magic&page=1&view=grid for buying options".to_string(),
            Self::Tix => "Check https://www.cardhoarder.com/ for buying options".to_string()
        }
    }
}

pub(crate) fn filter_cards(cards: Vec<Card>) -> Vec<DbCard> {
    let filtered: Vec<DbCard> = cards
        .into_iter()
        .filter_map(|card| {
            let prices = &card.prices;
            let no_price = prices.values().into_iter().all(|p| p.is_none());
            if no_price {
                return None;
            }

            Some(card.to_db_entry())
        })
        .collect();

    filtered
}
