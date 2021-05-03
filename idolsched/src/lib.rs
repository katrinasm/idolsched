pub mod mapdb;

pub mod acct_info;
pub mod card;
pub mod accessory;
pub mod schedule;
pub mod skill;
mod live_show;

use std::collections::BTreeMap;
use card_consumer::json_card::TrimCard;
use card::Card;
use accessory::Acc;
use acct_info::AcctInfo;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayGlob {
    pub song: mapdb::Song,
    pub album: Vec<card::Card>,
    pub inventory: Vec<accessory::Acc>,
}

impl PlayGlob {
    pub fn est_voltage(&self, sched: &schedule::Schedule, status: &mut live_show::Status) -> f64 {
        live_show::run(&self.song, &self.album, &self.inventory, sched, status)
    }
}

pub struct ScheduleDisplayInfo {
    pub voltage: f64,
    pub cards: [u32; 9], // ordinals, RRRGGGBBB
    pub sp3: [u32; 3], // indexes into `cards` (as u32)
    pub accs: [u32; 9], // indexes into the account's info (as u32)
}

pub fn init_acct(acct_json: &str) -> Result<AcctInfo, serde_json::Error> {
    let mut acct: AcctInfo = serde_json::from_str(acct_json)?;
    acct.force_valid();
    Ok(acct)
}

pub fn init_glob(card_details: &BTreeMap<u32, TrimCard>, acct: &AcctInfo, song_id: u32, song_json: &str)
-> Result<PlayGlob, serde_json::Error> {
    let song = mapdb::parse_song(song_id, song_json)?;
    let mut album = Vec::new();
    for (ordinal, jcard) in card_details.iter() {
        if let Some(card_inf) = acct.album.get(ordinal) {
            let card = Card::instantiate_json(&jcard, card_inf.lb, card_inf.idolized);
            album.push(card);
        }
    }
    let inventory = acct.accs.iter().map(|info| Acc::from_info(info)).collect();
    Ok(PlayGlob { album, inventory, song })
}

