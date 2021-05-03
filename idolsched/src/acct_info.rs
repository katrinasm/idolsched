use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use sifas_data::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct AcctInfo {
    pub bond: BTreeMap<Idol, BondInfo>,
    pub album: BTreeMap<u32, CardInfo>,
    pub accs: Vec<AccInfo>,
}

impl AcctInfo {
    pub fn force_valid(&mut self) {
        if self.album.len() < 9 {
            self.add_r1s()
        }
        self.pad_accs();
    }

    fn add_r1s(&mut self) {
        let r1_ordinals = &[
            1,  5,  9, 13, 17, 21, 25, 29, 33,
            37, 41, 45, 49, 53, 57, 61, 65, 69,
            73, 76, 79, 82, 85, 88, 91, 94, 97,
        ];
        for &ordinal in r1_ordinals {
            if let None = self.album.get(&ordinal) {
                self.album.insert(ordinal, CardInfo { lb: 0, idolized: false });
            }
        }
    }

    fn pad_accs(&mut self) {
        let needed = (9usize).saturating_sub(self.accs.len());
        for _ in 0 .. needed {
            self.accs.push(FILLER_ACC.clone());
        }
    }

    pub fn card_ordinals(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for &ordinal in self.album.keys() {
            v.push(ordinal);
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
    pub lb: u8,
    pub idolized: bool,
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
    Empty,
    Brooch, Keychain,
    Bracelet, Hairpin,
    Necklace, Earring,
    Pouch, Ribbon,
    Wristband, Towel,
    Bangle, Choker, Belt,
}

const FILLER_ACC: AccInfo = AccInfo {
    attribute: Attribute::Neutral,
    kind: AccKind::Empty,
    rarity: Rarity::R,
    lb: 0,
    lv: 1,
    sl: 1,
};

