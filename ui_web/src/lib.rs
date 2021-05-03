use std::collections::BTreeMap;

use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::console;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use idolsched::schedule::Schedule;
use idolsched::accessory::Acc;
use idolsched::card::Card;
use local_search::{SimpleIterSolver, anneal};
use card_consumer::json_card::TrimCard;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Serialize)]
pub struct ScheduleDisplayInfo {
    voltage: f64,
    cards: [u32; 9], // ordinals, RRRGGGBBB
    sp3: [u32; 3], // indexes into `cards` (as u32)
    accs: [u32; 9], // indexes into the account's info (as u32)
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    Ok(())
}

#[wasm_bindgen]
pub fn run_solver(steps: u32, json_cards: &str, acct_json: &str, song_id: u32, song_json: &str, rng_lo: u32, rng_hi: u32)
-> String {
    let acct = idolsched::init_acct(&acct_json).unwrap();
    let card_details = parse_cards(json_cards);
    let glob = idolsched::init_glob(&card_details, &acct, song_id, &song_json).unwrap();
    let rng_seed = (rng_hi as u64) << 32 | rng_lo as u64;
    let mut rng = SmallRng::seed_from_u64(rng_seed);
    let s0 = Schedule::new_random(&mut rng, glob.album.len(), glob.inventory.len());
    let pm = anneal::Params { rng, t0: 10_000.0, alpha: 1.0 - (1.0/65_536.0) };
    let mut annealer = anneal::Annealer::org(s0, glob.clone(), pm);
    let (final_sched, energy) = local_search::search_n(&mut annealer, steps).unwrap();

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

fn parse_cards(json_cards: &str) -> BTreeMap<u32, TrimCard> {
    serde_json::from_str(json_cards).unwrap()
}

