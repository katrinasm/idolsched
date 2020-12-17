use serde::{Deserialize, Serialize};
use serde_repr::*;
use crate::cards_api::enums::*;

// `kt_` prefixes are temporary workaround-y things until i get
// something to actually read songs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub default_attribute: Attribute,
    pub target_voltage: u32,
    pub lose_at_death: bool,
    pub sp_gauge_length: u32,
    pub note_stamina_reduce: u32,
    pub note_voltage_upper_limit: u32,
    pub collabo_voltage_upper_limit: u32,
    pub skill_voltage_upper_limit: u32,
    pub squad_change_voltage_upper_limit: u32,
    pub kt_notes: usize,
}

#[cfg(feature="cli")]
#[allow(dead_code)]
pub fn fetch_song(id: u32) -> Result<Song, crate::error::Error> {
    let path = format!("./mapdb/{}.json", id);
    let json = std::fs::read_to_string(path)?;
    parse_song(id, &json)
}

pub fn parse_song(id: u32, json: &str) -> Result<Song, crate::error::Error> {
    let live_type = (id / 1_0_000_00_0) % 10;
    let is_adv_plus = (id / 0_0_000_01_0) % 100 > 30;
    let jsong: JsonSong = serde_json::from_str(&json)?;
    println!("Beatmap: {}", jsong.song_name);
    let sp_gauge_length = match jsong.song_difficulty {
        Difficulty::Beginner => 3600,
        Difficulty::Intermediate => 4800,
        Difficulty::Advanced if !is_adv_plus => 6000,
        Difficulty::Advanced => 7200,
    };
    let (
        note_voltage_upper_limit,
        collabo_voltage_upper_limit,
        skill_voltage_upper_limit,
        squad_change_voltage_upper_limit
    ) = if !is_adv_plus {
        (50_000, 250_000, 50_000, 30_000)
    } else {
        (150_000, 500_000, 50_000, 30_000)
    };
    Ok(Song {
        default_attribute: jsong.song_attribute,
        target_voltage: jsong.ranks.rank_s,
        lose_at_death: live_type < 4,
        sp_gauge_length,
        note_stamina_reduce: jsong.note_damage,
        note_voltage_upper_limit,
        collabo_voltage_upper_limit,
        skill_voltage_upper_limit,
        squad_change_voltage_upper_limit,
        kt_notes: jsong.notes.len(),
    })
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
struct JsonSong {
    display_order: u32,
    song_name: String,
    song_attribute: Attribute,
    song_difficulty: Difficulty,
    ranks: JsonRankReqs,
    note_damage: u32,
    notes: Vec<JsonNote>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
enum Difficulty {
    Beginner = 10,
    Intermediate = 20,
    Advanced = 30,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
struct JsonRankReqs {
    #[serde(rename="S")] rank_s: u32,
    #[serde(rename="A")] rank_a: u32, // unused
    #[serde(rename="B")] rank_b: u32, // unused
    #[serde(rename="C")] rank_c: u32, // unused
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
struct JsonNote {
    pub time: u32,
    pub gimmick: Option<usize>,
}

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
// struct JsonNoteGimmick {
    // // technically this is an enum,
    // // but we just use it to index an array later
    // pub target: usize,
    // pub effect_type: SkillEffectType,
    // pub effect_amount: u32,
    // pub trigger: NoteGimmickTrigger,
// }

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize_repr, Serialize_repr)]
// enum JsonNoteGimmickTrigger {
    // Success = 1,
    // Failure = 2,
    // Always = 3,
// }

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
// struct JsonAcGimmick {
    // // technically this is an enum,
    // // but we just use it to index an array later
    // pub target: usize,
    // pub effect_type: SkillEffectType,
    // pub effect_amount: u32,
    // pub trigger: AcGimmickTrigger,
// }

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize_repr, Serialize_repr)]
// enum JsonAcGimmickTrigger {
    // Start = 1,
    // Success = 2,
    // Failure = 3,
    // End = 4,
// }
