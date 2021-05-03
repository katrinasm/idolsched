pub mod json_card;
#[cfg(feature = "reqwest")]
pub mod network;
mod error;

pub use error::Error;

use std::collections::BTreeMap;
use json_card::{JsonCard, TrimCard};
use sifas_data::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Monicker {
    pub idol: Idol,
    pub rarity: Rarity,
    pub count: u32,
}

impl std::fmt::Display for Monicker {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.rarity {
            Rarity::R => write!(f, "{:?} R{}", self.idol, self.count),
            Rarity::Sr => write!(f, "{:?} SR{}", self.idol, self.count),
            Rarity::Ur => write!(f, "{:?}{}", self.idol, self.count),
        }
    }
}

impl Monicker {
    pub fn asset_lemma(&self) -> String {
        let mut name = format!("{:?}", self.idol);
        name.make_ascii_lowercase();
        name += match self.rarity {
            Rarity::R => "-r",
            Rarity::Sr => "-sr",
            Rarity::Ur => "-ur",
        };
        name += &self.count.to_string();
        name
    }
}

pub fn assign_names(cards: &BTreeMap<u32, JsonCard>) -> BTreeMap<u32, Monicker> {
    let mut output_names = BTreeMap::new();
    let mut counters: BTreeMap<(Idol, Rarity), u32> = BTreeMap::new();
    for &idol in sifas_data::ALL_IDOLS {
        counters.insert((idol, Rarity::R), 1);
        counters.insert((idol, Rarity::Sr), 1);
        counters.insert((idol, Rarity::Ur), 1);
    }

    for (&sin, jcard) in cards.iter() {
        let (idol, rarity) = (jcard.member, jcard.rarity);
        let count = *counters.get(&(idol, rarity)).unwrap();
        counters.insert((idol, rarity), count + 1);
        output_names.insert(sin, Monicker {idol, rarity, count});
    }

    output_names
}

pub fn trim_cards(cards: &BTreeMap<u32, JsonCard>) -> BTreeMap<u32, TrimCard> {
    let mut trimmed = BTreeMap::new();
    for (&ord, card) in cards.iter() {
        trimmed.insert(ord, card.trim());
    }
    trimmed
}

