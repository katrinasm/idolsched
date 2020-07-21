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

pub async fn get_cards(cfg: &Cfg, mut id_list: Vec<u32>) -> Result<HashMap<u32, json_card::JsonCard>, Error> {
    let mut cachedata = cache::load_cache(cfg)?;
    let mut output_cards = HashMap::new();
    id_list.retain(|id| if let Some(card_json) = cachedata.cards.get(&id) {
        output_cards.insert(*id, card_json.clone());
        false
    } else {
        true
    });
    if id_list.len() != 0 {
        println!("Requesting card data from {}", cfg.provider);
        let client = reqwest::Client::new();
        let mut response: json_card::JsonCardRq = {
            let res = client.get(&(cfg.provider.clone() + "id/" + &id_list_name(&id_list)))
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
            let id = card.id;
            output_cards.insert(id, card.clone());
            cachedata.cards.insert(id, card);
        }
        cache::save_cache(cfg, &cachedata)?;
    }
    Ok(output_cards)
}

fn id_list_name(id_list: &Vec<u32>) -> String {
    let mut s = String::with_capacity(10 * id_list.len() + 4);
    for i in 0 .. id_list.len() {
        s += &format!("{}", id_list[i]);
        if i + 1 != id_list.len() {
            s.push(',');
        } else {
            s += ".json";
        }
    }
    s
}

