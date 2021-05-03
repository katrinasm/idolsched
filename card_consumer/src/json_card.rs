use sifas_data::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub member: Idol,
    pub role_effect: RoleEffect,
    pub normal_appearance: JsonAppearance,
    pub idolized_appearance: JsonAppearance,
    pub active_skill: JsonSkill,
    pub passive_skills: Vec<JsonSkill>,
    pub idolized_offset: [u32; 4], // level, appeal, stamina, technique
    pub tt_offset: Vec<[u32; 4]>,  // level, appeal, stamina, technique
    pub stats: Vec<[u32; 4]>,      // level, appeal, stamina, technique
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct TrimCard {
    pub ordinal: u32,
    pub rarity: Rarity,
    pub max_level: u32,
    pub attribute: Attribute,
    pub role: Role,
    pub member: Idol,
    pub role_effect: RoleEffect,
    pub active_skill: JsonSkill,
    pub passive_skills: Vec<JsonSkill>,
    pub idolized_offset: [u32; 4],
    pub tt_offset: Vec<[u32; 4]>,
    pub stats: Vec<[u32; 4]>, // skips 1 .. default_lv(self.rarity)
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

    pub fn trim(&self) -> TrimCard {
        let mut stats = Vec::new();
        stats.push(self.stats[0]);
        let lv = (default_lv(self.rarity) - 1) as usize;
        for i in lv .. lv + 6 {
            stats.push(self.stats[i]);
        }
        TrimCard {
            ordinal: self.ordinal,
            rarity: self.rarity,
            max_level: self.max_level,
            attribute: self.attribute,
            role: self.role,
            member: self.member,
            role_effect: self.role_effect,
            active_skill: self.active_skill.clone(),
            passive_skills: self.passive_skills.clone(),
            idolized_offset: self.idolized_offset,
            tt_offset: self.tt_offset.clone(),
            stats,
        }
    }
}

impl TrimCard {
    pub fn stats_with(&self, level: Option<u8>, fed: bool, lb: u8) -> [u32; 4] {
        assert!(lb < 6);
        let level_i = if let Some(n) = level {
            assert!(n != 0 && n < 100);
            n.saturating_sub(default_lv(self.rarity) - 1) as usize
        } else {
            1
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

fn default_lv(rarity: Rarity) -> u8 {
    match rarity {
        Rarity::R => 40,
        Rarity::Sr => 60,
        Rarity::Ur => 80,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonAppearance {
    pub name: String,
    pub image_asset_path: String,
    pub thumbnail_asset_path: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JsonSkill {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub skill_type: Option<u32>,
    pub sp_gauge_point: Option<u32>,
    pub rarity: u32,
    pub trigger_type: Option<LivePassiveSkillTrigger>,
    pub trigger_probability: u32,
    pub target: JsonTarget,
    // pub conditions: Vec<JsonIdk>,
    pub levels: Vec<JsonSkillData>,
    pub levels_2: Option<Vec<JsonSkillData>>,
}

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
#[serde(from = "SkillDataTuple", into = "SkillDataTuple")]
pub struct JsonSkillData {
    pub target_parameter: u32,
    pub effect_type: SkillEffectType,
    pub effect_value: u32,
    pub scale_type: u8,
    pub calc_type: u8,
    pub timing: SkillTiming,
    pub finish_type: SkillEffectFinishTimingType,
    pub finish_value: u32,
}

type SkillDataTuple = (u32, SkillEffectType, u32, u8, u8, SkillTiming, SkillEffectFinishTimingType, u32);

impl From<SkillDataTuple> for JsonSkillData {
    fn from(tup: SkillDataTuple) -> JsonSkillData {
        JsonSkillData {
            target_parameter: tup.0,
            effect_type: tup.1,
            effect_value: tup.2,
            scale_type: tup.3,
            calc_type: tup.4,
            timing: tup.5,
            finish_type: tup.6,
            finish_value: tup.7,
        }
    }
}

impl Into<SkillDataTuple> for JsonSkillData {
    fn into(self) -> SkillDataTuple {
        (self.target_parameter, self.effect_type, self.effect_value, self.scale_type, self.calc_type, self.timing, self.finish_type, self.finish_value)
    }
}

