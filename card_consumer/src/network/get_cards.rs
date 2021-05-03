use std::collections::BTreeMap;

use crate::{Error, Monicker, json_card, assign_names};
use super::{cache, Cfg};

pub async fn id_list(cfg: &Cfg) -> Result<Vec<json_card::JsonOrdId>, Error> {
    let text = reqwest::get(&(cfg.provider.clone() + "id_list.json")).await?.text().await?;
    let parsed: json_card::JsonIdRq = serde_json::from_str(&text)?;
    Ok(parsed.result)
}

pub async fn til_latest(cfg: &Cfg)
-> Result<(BTreeMap<u32, json_card::JsonCard>, BTreeMap<u32, Monicker>), Error> {
    let ordinals: Vec<u32> = id_list(cfg).await?.iter().map(|row| row.ordinal).collect();
    by_ordinal(cfg, ordinals).await
}

pub async fn by_ordinal(cfg: &Cfg, ordinal_list: Vec<u32>)
-> Result<(BTreeMap<u32, json_card::JsonCard>, BTreeMap<u32, Monicker>), Error> {
    let mut cachedata = cache::load_cache(cfg)?;
    let mut output_cards = BTreeMap::new();

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
            let mut req = client.get(&(cfg.provider.clone() + "ordinal/" + &ordinal_list_name(&fetch_list)));
            if let Some(ref lang) = cfg.language {
                req = req.header("Accept-Language", lang);
            }
            let res = req.send().await?;
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

    let output_names = assign_names(&output_cards);

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
