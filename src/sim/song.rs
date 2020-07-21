use super::basic_data::Attribute;

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
    pub kt_notes: u32,
}

// NEO adv
pub const TEST_SONG: Song = Song {
    default_attribute: Attribute::Cool,
    target_voltage: 2_485_100,
    lose_at_death: true,
    sp_gauge_length: 6000,
    note_stamina_reduce: 280,
    note_voltage_upper_limit: 50_000,
    collabo_voltage_upper_limit: 250_000,
    skill_voltage_upper_limit: 50_000,
    squad_change_voltage_upper_limit: 30_000,
    kt_notes: 157,
};

