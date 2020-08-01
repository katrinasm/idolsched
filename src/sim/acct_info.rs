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
        let r1_ordinals = &[
            1,  5,  9, 13, 17, 21, 25, 29, 33,
            37, 41, 45, 49, 53, 57, 61, 65, 69,
            73, 76, 79, 82, 85, 88, 91, 94, 97,
        ];
        'r1s: for &ordinal in r1_ordinals {
            for card in self.cards.iter() {
                if card.ordinal == ordinal {
                    continue 'r1s;
                }
            }
            self.cards.push(CardInfo { ordinal, lb: 0, fed: false })
        }
    }

    pub fn card_ordinals(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for card in self.cards.iter() {
            v.push(card.ordinal);
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
    pub ordinal: u32,
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
