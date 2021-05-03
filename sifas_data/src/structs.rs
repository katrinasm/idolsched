use serde::{Deserialize, Serialize};
use super::misc_enums::*;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub struct RoleEffect {
    pub change_effect_type: Role,
    pub change_effect_value: u32,
    pub positive_type: Role,
    pub positive_value: u32,
    pub negative_type: Role,
    pub negative_value: u32,
}

