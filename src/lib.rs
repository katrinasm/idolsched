/*#![allow(dead_code, unused_imports)]
pub mod error;
pub mod sim;
mod cards_api;
mod anneal;
mod state;
mod mapdb;

use serde::{Serialize};
use std::collections::{HashMap, BTreeMap};
use wasm_bindgen::prelude::*;

use cards_api::json_card::JsonCard;
use rand::SeedableRng;
use rand::rngs::SmallRng as Rng;
use sim::card::Card;
use sim::acct_info::{AcctInfo, AccInfo, AccKind};
use sim::schedule::Schedule;

#[derive(Serialize)]
pub struct ScheduleDisplayInfo {
    voltage: f64,
    cards: [u32; 9], // ordinals, RRRGGGBBB
    sp3: [u32; 3], // indexes into `cards` (as u32)
    accs: [u32; 9], // indexes into the account's info (as u32)
}

#[wasm_bindgen]
pub fn solve(cards_json: &str, acct_json: &str, song_id: u32, song_json: &str, steps: u32, rng_lo: u32, rng_hi: u32) -> String {
    let rng_seed = (rng_hi as u64) << 32 | rng_lo as u64;
    let song = mapdb::parse_song(song_id, song_json).unwrap();
    let acct = get_acct(acct_json);
    let card_details = get_jcards(cards_json);
    let mut struggle_map: HashMap<u32, sim::acct_info::CardInfo> = HashMap::new();
    for card_inf in acct.cards.iter() {
        struggle_map.insert(card_inf.ordinal, *card_inf);
    }
    let mut album = Vec::new();
    for (ordinal, jcard) in card_details.iter() {
        if let Some(card_inf) = struggle_map.get(ordinal) {
            let card = sim::card::Card::instantiate_json(&jcard, card_inf.lb, card_inf.fed);
            album.push(card);
        }
    }
    let inventory = acct.accs.iter().map(|info| sim::accessory::Acc::from_info(info)).collect();
    let glob = sim::PlayGlob { album, inventory, song };
    let mut rng = Rng::seed_from_u64(rng_seed);
    let s0 = sim::schedule::Schedule::new_random(&mut rng, glob.album.len(), glob.inventory.len());
    let (_steps, final_sched, energy) = anneal::anneal(&mut rng, &s0, &glob, steps, 10_000.0);
    serde_json::to_string(&sdi(&glob.album, final_sched, energy)).unwrap()
}

fn sdi(album: &[Card], sched: Schedule, energy: f64) -> ScheduleDisplayInfo {
    let mut cards = [0; 9];
    for (i, card_i) in sched.cards.iter().enumerate() {
        cards[i] = album[*card_i].ordinal;
    }
    let mut sp3 = [0; 3];
    for (i, v) in sched.sp3.iter().enumerate() {
        sp3[i] = *v as u32;
    }
    let mut accs = [0; 9];
    for (i, v) in sched.accs.iter().enumerate() {
        accs[i] = *v as u32;
    }
    let voltage = -energy;
    ScheduleDisplayInfo { voltage, cards, sp3, accs }
}

fn get_acct(acct_json: &str) -> AcctInfo {
    let mut acct: AcctInfo = serde_json::from_str(acct_json).unwrap();
    acct.force_valid();
    acct
}

fn get_jcards(cards_json: &str) -> BTreeMap<u32, JsonCard> {
    serde_json::from_str(cards_json).unwrap()
}

*/