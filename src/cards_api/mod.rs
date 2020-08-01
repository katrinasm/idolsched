pub mod json_card;
pub mod api_cfg;
pub mod error;
mod skill_enums;
mod wave_enums;
mod misc_enums;
mod cache;

pub use api_cfg::Cfg;
pub use error::Error;

use std::collections::HashMap;

pub mod enums {
    pub use super::skill_enums::*;
    pub use super::wave_enums::*;
    pub use super::misc_enums::*;
}

pub async fn get_cards(cfg: &Cfg, mut ordinal_list: Vec<u32>) -> Result<HashMap<u32, json_card::JsonCard>, Error> {
    let mut cachedata = cache::load_cache(cfg)?;
    let mut output_cards = HashMap::new();
    ordinal_list.retain(|ordinal| if let Some(card_json) = cachedata.cards.get(&ordinal) {
        output_cards.insert(*ordinal, card_json.clone());
        false
    } else {
        true
    });
    if ordinal_list.len() != 0 {
        println!("Requesting card data from {}", cfg.provider);
        let client = reqwest::Client::new();
        let mut response: json_card::JsonCardRq = {
            let res = client.get(&(cfg.provider.clone() + "ordinal/" + &ordinal_list_name(&ordinal_list)))
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
    Ok(output_cards)
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

