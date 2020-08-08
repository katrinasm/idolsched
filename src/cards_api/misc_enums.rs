// enums not stolen from kirara
use serde_repr::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Role {
    Vo = 1,
    Sp = 2,
    Gd = 3,
    Sk = 4,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum Idol {
    Honoka = 1,
    Eli = 2,
    Kotori = 3,
    Umi = 4,
    Rin = 5,
    Maki = 6,
    Nozomi = 7,
    Hanayo = 8,
    Nico = 9,
    Chika = 101,
    Riko = 102,
    Kanan = 103,
    Dia = 104,
    You = 105,
    Yohane = 106,
    Hanamaru = 107,
    Mari = 108,
    Ruby = 109,
    Ayumu = 201,
    Kasumi = 202,
    Shizuku = 203,
    Karin = 204,
    Ai = 205,
    Kanata = 206,
    Setsuna = 207,
    Emma = 208,
    Rina = 209,
    Shioriko = 210,
}
use Idol::*;
pub const ALL_IDOLS: &[Idol] = &[
    Honoka, Eli, Kotori, Umi, Rin, Maki, Nozomi, Hanayo, Nico,
    Chika, Riko, Kanan, Dia, You, Yohane, Hanamaru, Mari, Ruby,
    Ayumu, Kasumi, Shizuku, Karin, Ai, Kanata, Setsuna, Emma, Rina
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Attribute {
    Neutral = 0, // this is just a guess tbh
    Smile = 1,
    Pure = 2,
    Cool = 3,
    Active = 4,
    Natural = 5,
    Elegant = 6,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Rarity {
    R = 10,
    Sr = 20,
    Ur = 30,
}

// FIXME: i didn't actually check this in the database lmao
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SkillTiming {
    Normal = 1,
    Attacking = 2,
}