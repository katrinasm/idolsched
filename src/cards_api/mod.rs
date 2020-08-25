pub mod json_card;
pub mod api_cfg;
pub mod error;
mod skill_enums;
mod wave_enums;
mod misc_enums;
mod cache;

pub use api_cfg::Cfg;
pub use error::Error;

use std::collections::{HashMap, BTreeMap};

pub mod enums {
    pub use super::skill_enums::*;
    pub use super::wave_enums::*;
    pub use super::misc_enums::*;
}

use misc_enums::{Idol, Rarity};

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

pub async fn get_cards(cfg: &Cfg, ordinal_list: Vec<u32>) -> Result<(BTreeMap<u32, json_card::JsonCard>, BTreeMap<u32, Monicker>), Error> {
    let mut cachedata = cache::load_cache(cfg)?;
    let mut output_cards = BTreeMap::new();
    let mut output_names = BTreeMap::new();

    let max_ordinal = ordinal_list.iter().cloned().max().unwrap_or(0);
    let mut fetch_list: Vec<u32> = (1 ..= max_ordinal).collect();

    fetch_list.retain(|ordinal| if let Some(card_json) = cachedata.cards.get(&ordinal) {
        output_cards.insert(*ordinal, card_json.clone());
        false
    } else {
        true
    });

    if fetch_list.len() != 0 {
        println!("Requesting card data from {}", cfg.provider);
        let client = reqwest::Client::new();
        let mut response: json_card::JsonCardRq = {
            let res = client.get(&(cfg.provider.clone() + "ordinal/" + &ordinal_list_name(&fetch_list)))
                .send().await?;
            let ra = res.remote_addr();
            let status = res.status();
            let text = res.text().await?;
            match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(e) => {
                    println!("{}: HTTP {}", ra.map(|x| format!("{}", x)).unwrap_or(String::from("<???>")), status);
                    println!("response body:\n{}", text);
                    return Err(e.into());
                },
            }
        };
        for card in response.result.drain(..) {
            let ordinal = card.ordinal;
            output_cards.insert(ordinal, card.clone());
            cachedata.cards.insert(ordinal, card);
        }
        cache::save_cache(cfg, &cachedata)?;
    }

    let mut counters: HashMap<(Idol, Rarity), u32> = HashMap::with_capacity(misc_enums::ALL_IDOLS.len() * 3);
    for &idol in misc_enums::ALL_IDOLS {
        counters.insert((idol, Rarity::R), 1);
        counters.insert((idol, Rarity::Sr), 1);
        counters.insert((idol, Rarity::Ur), 1);
    }

    for (&sin, jcard) in output_cards.iter() {
        let (idol, rarity) = (jcard.member, jcard.rarity);
        let count = *counters.get(&(idol, rarity)).unwrap();
        counters.insert((idol, rarity), count + 1);
        output_names.insert(sin, Monicker {idol, rarity, count});
    }

    Ok((output_cards, output_names))
}

fn ordinal_list_name(ordinal_list: &Vec<u32>) -> String {
    let mut s = String::with_capacity(10 * ordinal_list.len() + 4);
    for i in 0 .. ordinal_list.len() {
        s += &format!("{}", ordinal_list[i]);
        if i + 1 != ordinal_list.len() {
            s.push(',');
        } else {
            s += ".json";
        }
    }
    s
}

