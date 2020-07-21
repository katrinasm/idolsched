use serde::{Deserialize, Serialize};
use super::skill_enums::*;
use super::misc_enums::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
pub struct JsonIdRq {
    pub result: Vec<JsonOrdId>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Deserialize, Serialize)]
pub struct JsonOrdId {
    pub ordinal: u32,
    pub id: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonCardRq {
    pub result: Vec<JsonCard>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonCard {
    pub id: u32,
    pub ordinal: u32,
    pub rarity: Rarity,
    pub max_level: u32,
    pub attribute: Attribute,
    pub role: Role,
    pub training_tree_m_id: u32,
    pub sp_point: u32,
    pub exchange_item_id: u32,
    pub max_passive_skill_slot: u32,
    pub background_asset_path: String,
    pub member: Idol,
    pub role_effect: JsonRoleEffect,
    pub normal_appearance: JsonAppearance,
    pub idolized_appearance: JsonAppearance,
    pub active_skill: JsonSkill,
    pub passive_skills: Vec<JsonSkill>,
    pub idolized_offset: [u32; 4], // level, appeal, stamina, technique
    pub tt_offset: Vec<[u32; 4]>,  // level, appeal, stamina, technique
    pub stats: Vec<[u32; 4]>,      // level, appeal, stamina, technique
    pub costume_info: Option<(
        String,
        u32,
        Option<JsonIdk>,
        Option<JsonIdk>,
    )>,
}

impl JsonCard {
    pub fn stats_with(&self, level: Option<u8>, fed: bool, lb: u8) -> [u32; 4] {
        assert!(lb < 6);
        let level_i = if let Some(n) = level {
            assert!(n != 0 && n < 100);
            (n - 1) as usize
        } else {
            match self.rarity {
                Rarity::R => 40 - 1,
                Rarity::Sr => 60 - 1,
                Rarity::Ur => 80 - 1,
            }
        };
        let mut stats = self.stats[level_i];
        if fed {
            let idlz_ofs = self.idolized_offset;
            stats[1] += idlz_ofs[1]; stats[2] += idlz_ofs[2]; stats[3] += idlz_ofs[3];
            let lb_ofs = self.tt_offset[lb as usize];
            stats[1] += lb_ofs[1]; stats[2] += lb_ofs[2]; stats[3] += lb_ofs[3];
        }
        stats
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub struct JsonRoleEffect {
    pub change_effect_type: Role,
    pub change_effect_value: u32,
    pub positive_type: Role,
    pub positive_value: u32,
    pub negative_type: Role,
    pub negative_value: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonAppearance {
    name: String,
    image_asset_path: String,
    thumbnail_asset_path: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonSkill {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub skill_type: Option<u32>,
    pub sp_gauge_point: Option<u32>,
    pub icon_asset_path: Option<String>,
    pub rarity: u32,
    pub trigger_type: Option<LivePassiveSkillTrigger>,
    pub trigger_probability: u32,
    pub target: JsonTarget,
    pub conditions: Vec<JsonIdk>,
    pub levels: Vec<SkillData>,
}

// target_parameter, effect_type, effect_value, scale_type,
// calc_type,        timing,      finish_type,  finish_value
pub type SkillData = (u32, SkillEffectType, u32, u8, u8, SkillTiming, SkillEffectFinishTimingType, u32);

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonTarget {
    pub id: u32,
    pub self_only: u32,
    pub not_self: u32,
    pub apply_count: u32,
    pub owner_party: u32,
    pub owner_school: u32,
    pub owner_year: u32,
    pub owner_subunit: u32,
    pub owner_attribute: u32,
    pub owner_role: u32,
    pub fixed_attributes: Vec<Attribute>,
    pub fixed_members: Vec<Idol>,
    pub fixed_subunits: Vec<u32>,
    pub fixed_schools: Vec<u32>,
    pub fixed_years: Vec<u32>,
    pub fixed_roles: Vec<Role>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonIdk {}
