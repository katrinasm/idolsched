pub mod misc_enums;
pub mod skill_enums;
pub mod wave_enums;
pub mod structs;

use misc_enums::Idol;
use misc_enums::Idol::*;

// convenience function for sifas percentile values,
// represented such that 1.0 (100%) == 10_000
pub fn pct<T: Into<f64>>(x: T) -> f64 {
    x.into() / 10_000f64
}

pub const ALL_IDOLS: &[Idol] = &[
    Honoka, Eli, Kotori, Umi, Rin, Maki, Nozomi, Hanayo, Nico,
    Chika, Riko, Kanan, Dia, You, Yohane, Hanamaru, Mari, Ruby,
    Ayumu, Kasumi, Shizuku, Karin, Ai, Kanata, Setsuna, Emma, Rina, Shioriko
];

pub mod prelude {
    pub use super::misc_enums::*;
    pub use super::skill_enums::*;
    pub use super::wave_enums::*;
    pub use super::structs::RoleEffect;
    pub use super::pct;
}

