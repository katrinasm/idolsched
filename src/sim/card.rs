use super::skill::{Skill, mask_for};
use super::basic_data::*;
use crate::cards_api::json_card::JsonCard;

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub ordinal: u32, // mainly used for display
    pub member: Idol,
    pub rarity: Rarity,
    pub attribute: Attribute,
    pub role: Role,
    pub role_effect: RoleEffect,
    pub level: u32,
    pub appeal: u32,
    pub technique: u32,
    pub stamina: u32,
    pub crit_rate_bonus: bool,
    pub skill_mask: u64,
    pub tap_skill: Skill,
    pub etc_skills: Vec<Skill>,
}

impl Card {
    pub fn instantiate_json(jc: &JsonCard, lb: u8, fed: bool) -> Card {
        use Rarity::*;
        let [level, appeal, technique, stamina] = jc.stats_with(None, fed, lb);
        let (tap_lv, etc_lv) = if !fed { (1, 1) } else {
            match (jc.rarity, lb) {
                (_, 0)  => (3, 3),
                (R, 1)  => (3, 3),
                (R, 2)  => (3, 3),
                (R, 3)  => (4, 3),
                (R, 4)  => (4, 4),
                (Sr, 1) => (3, 3),
                (Sr, 2) => (3, 3),
                (Sr, 3) => (4, 4),
                (Sr, 4) => (4, 4),
                (Ur, 1) => (3, 4),
                (Ur, 2) => (3, 4),
                (Ur, 3) => (4, 4),
                (Ur, 4) => (4, 4),
                (_, 5)  => (5, 5),
                _ => panic!("bad lb somehow"),
            }
        };
        let tap_skill = Skill::from_json(&jc.active_skill, tap_lv, Some(jc));
        let etc_skills: Vec<Skill> = jc.passive_skills.iter()
            .map(|s| Skill::from_json(s, etc_lv, Some(jc)))
            .collect();
        let skill_mask = mask_for(jc);
        Card {
            ordinal: jc.ordinal,
            member: jc.member,
            rarity: jc.rarity,
            attribute: jc.attribute,
            role: jc.role,
            role_effect: jc.role_effect,
            crit_rate_bonus: technique > appeal && technique > stamina,
            skill_mask,
            tap_skill,
            etc_skills,
            level, appeal, technique, stamina,
        }
    }

    pub fn display_name(&self) -> String {
        let rarity = match self.rarity {
            Rarity::R => "R",
            Rarity::Sr => "SR",
            Rarity::Ur => "UR",
        };
        format!("{:>3} {} {:?}", self.ordinal, rarity, self.member)
    }
}

