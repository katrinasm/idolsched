use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use super::basic_data::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct AcctInfo {
    pub bond: BTreeMap<Idol, BondInfo>,
    pub cards: Vec<CardInfo>,
    pub accs: Vec<AccInfo>,
}

impl AcctInfo {
    pub fn force_valid(&mut self) {
        if self.cards.len() < 9 {
            self.add_r1s()
        }
    }

    fn add_r1s(&mut self) {
        let r1_ids = &[
            10_001_10_01, 10_002_10_01, 10_003_10_01,
            10_004_10_01, 10_005_10_01, 10_006_10_01,
            10_007_10_01, 10_008_10_01, 10_009_10_01,
            10_101_10_01, 10_102_10_01, 10_103_10_01,
            10_104_10_01, 10_105_10_01, 10_106_10_01,
            10_107_10_01, 10_108_10_01, 10_109_10_01,
            10_201_10_01, 10_202_10_01, 10_203_10_01,
            10_204_10_01, 10_205_10_01, 10_206_10_01,
            10_207_10_01, 10_208_10_01, 10_209_10_01
        ];
        'r1s: for &id in r1_ids {
            for card in self.cards.iter() {
                if card.id == id {
                    continue 'r1s;
                }
            }
            self.cards.push(CardInfo { id, lb: 0, fed: false })
        }
    }

    pub fn card_ids(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for card in self.cards.iter() {
            v.push(card.id);
        }
        v
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct BondInfo {
    pub bond_lv: u32,
    pub board_appeal: u32,
    pub board_stamina: u32,
    pub board_technique: u32,
}

impl Default for BondInfo {
    fn default() -> BondInfo {
        BondInfo { bond_lv: 1, board_appeal: 0, board_stamina: 0, board_technique: 0, }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct CardInfo {
    pub id: u32,
    pub lb: u8,
    pub fed: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct AccInfo {
    pub attribute: Attribute,
    // this is probably incorrect but will it cause a problem? probably not, huh
    pub kind: AccKind,
    pub rarity: Rarity,
    pub lb: u8,
    pub lv: u8,
    pub sl: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccKind {
    Brooch, Keychain,
    Bracelet, Hairpin,
    Necklace, Earring,
    Pouch, Ribbon,
    Wristband, Towel,
    Bangle, Choker, Belt,
}
