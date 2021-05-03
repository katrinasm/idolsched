// https://github.com/summertriangle-dev/arposandra/blob/master/libcard2/wave_cs_enums.py

use serde_repr::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LiveAppealTimeMission {
    GotVoltage = 1,
    JudgeSuccessGood = 2,
    JudgeSuccessGreat = 3,
    JudgeSuccessPerfect = 4,
    MaxVoltage = 5,
    TriggerSp = 6,
    UseCardUniq = 7,
}

