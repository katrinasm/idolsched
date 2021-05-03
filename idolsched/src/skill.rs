use sifas_data::prelude::*;
use card_consumer::json_card::{JsonSkill, TrimCard, JsonSkillData};

// target masks are:
// 63                             32
// ----~~~~----~~~~----~~~~----~~~~
// ----NNNNNNNNNNAAAAAAAAAMMMMMMMMM
// 31                             0
// ----~~~~----~~~~----~~~~----~~~~
// ----kpgv-xenacps-------876543210
// where:
// N, A, M are the niji/aqours/μ’s members, in order, honoka ... shioriko
// v, g, p, k: vo, gd, sp, sk
// xenacps: attributes (x neutral)
// digits: formation positions 0...8
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Skill {
    pub prob: f64,
    pub target_mask: u64,
    pub eff: SkillEff,
    jishin: bool, // 'self' is a keyword. technically i could use r#self i guess
    others: bool,
    strat: bool,
}

impl std::default::Default for Skill {
    fn default() -> Skill {
        Skill {
            prob: 0.0,
            target_mask: 0,
            eff: SkillEff::Unimplemented,
            jishin: false,
            others: false,
            strat: false,
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SkillEff {
    Unimplemented,
    AppealPlus(f64),
    StaminaPlus(f64),
    TechniquePlus(f64),
    TypePlus(f64),
    CritRatePlus(f64),
    CritPlus(f64),
    VoPlus(ValueType),
    Heal(ValueType),
    Shield(ValueType),
    Damage(ValueType),
    RemoveShield(ValueType),
    AppealUpAdd(f64, Duration),
    AppealUp(f64, Duration),
    AppealUpEx(f64, Duration),
    TapVoUp(f64, Duration),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueType {
    Constant(f64),
    CardAppeal(f64),
    CardStamina(f64),
    CardTechnique(f64),
    StamGauge(f64),
    Cardinal(f64, u64),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Duration {
    Permanent,
    Turn(u32),
    Immediate,
    WaveEnd,
    WaveSuccess,
    SpExecuteCount(u32),
    ChangeSquadCount(u32),
}

impl From<&JsonSkillData> for Duration {
    fn from(sd: &JsonSkillData) -> Duration {
        use SkillEffectFinishTimingType as Tt;
        match sd.finish_type {
            Tt::Permanent => Duration::Permanent,
            Tt::Turn => Duration::Turn(sd.finish_value),
            Tt::WaveEnd => Duration::WaveEnd,
            Tt::WaveSuccess => Duration::WaveSuccess,
            Tt::SpExecuteCount => Duration::SpExecuteCount(sd.finish_value),
            Tt::ChangeSquadCount => Duration::ChangeSquadCount(sd.finish_value),
            // unimplemented: Tt::Voltage, Tt::Non
            _ => Duration::Immediate,
        }
    }
}

impl Skill {
    pub fn from_json(js: &JsonSkill, lv: u8, owner: Option<&TrimCard>) -> Skill {
        let lv_i = js.levels.len().min(lv as usize) - 1;
        let eff = process_effect(&js.levels[lv_i]);
        // we are ignoring a couple of fixed targets right now
        // out of *plain laziness*.
        let jishin = js.target.self_only != 0;
        let others = js.target.not_self != 0;
        let strat = js.target.owner_party != 0;

        let mut target_mask = 0;
        if let Some(jc) = owner {
            if js.target.owner_school != 0 {
                target_mask |= school_mask(member_ofs(jc.member) / 9);
            }
            if js.target.owner_year != 0 {
                target_mask |= year_mask_m(jc.member);
            }
            if js.target.owner_subunit != 0 {
                target_mask |= subunit_mask_m(jc.member);
            }
            if js.target.owner_attribute != 0 {
                target_mask |= attribute_mask(jc.attribute);
            }
            if js.target.owner_role != 0 {
                target_mask |= role_mask(jc.role);
            }
        }
        for &member in js.target.fixed_members.iter() {
            target_mask |= member_mask(member);
        }
        for &attribute in js.target.fixed_attributes.iter() {
            target_mask |= attribute_mask(attribute);
        }
        for &role in js.target.fixed_roles.iter() {
            target_mask |= role_mask(role);
        }

        if !(jishin || others || strat) && target_mask == 0 && js.target.apply_count == 9 {
            target_mask = 0x1ff;
        }

        Skill {
            jishin, others, strat, target_mask,
            eff,
            prob: pct(js.trigger_probability),
        }
    }

    pub fn imbue_pos(&self, pos: usize) -> Skill {
        let mut target_mask = self.target_mask;
        if self.jishin {
            target_mask |= 1 << pos;
        }
        if self.others {
            target_mask |= 0x1ff ^ (1 << pos);
        }
        if self.strat {
            target_mask |= 7 << ((pos / 3) * 3);
        }
        Skill { target_mask, .. *self }
    }
}

pub fn mask_for(jc: &TrimCard) -> u64 {
    member_mask(jc.member) | role_mask(jc.role) | attribute_mask(jc.attribute)
}

fn process_effect(sd: &JsonSkillData) -> SkillEff {
    use SkillEff::*;
    use ValueType::*;
    use SkillEffectType::*;
    let (effect_type, effect_value) = (sd.effect_type, sd.effect_value);
    match effect_type {
        AddAppealBase            => AppealPlus(pct(effect_value)),
        AddStaminaBase           => StaminaPlus(pct(effect_value)),
        AddTechniqueBase         => TechniquePlus(pct(effect_value)),
        AddRoleMeritBuffBase     => TypePlus(pct(effect_value)),
        AddCriticalRateBase      => CritRatePlus(pct(effect_value)),
        AddCriticalAppealBase    => CritPlus(pct(effect_value)),

        AddShield => Shield(Constant(effect_value as f64)),
        HealLife => Heal(Constant(effect_value as f64)),
        AddDamage => Damage(Constant(effect_value as f64)),
        AddShieldByCardStamina   => Shield(CardStamina(pct(effect_value))),
        HealLifeByCardStamina    =>   Heal(CardStamina(pct(effect_value))),
        AddShieldByCardAppeal    => Shield(CardAppeal(pct(effect_value))),
        HealLifeByCardAppeal     =>   Heal(CardAppeal(pct(effect_value))),
        AddShieldByCardTechnique => Shield(CardTechnique(pct(effect_value))),
        HealLifeByCardTechnique  =>   Heal(CardTechnique(pct(effect_value))),
        AddShieldByMaxLife       => Shield(StamGauge(pct(effect_value))),
        HealLifeByMaxLife        =>   Heal(StamGauge(pct(effect_value))),

        HealLifeByNumOfVo        => Heal(Cardinal(effect_value as f64, role_mask(Role::Vo))),
        HealLifeByNumOfSp        => Heal(Cardinal(effect_value as f64, role_mask(Role::Sp))),
        HealLifeByNumOfSk        => Heal(Cardinal(effect_value as f64, role_mask(Role::Sk))),
        HealLifeByNumOfGd        => Heal(Cardinal(effect_value as f64, role_mask(Role::Gd))),

        AddDamageByNumOfVo       => Damage(Cardinal(effect_value as f64, role_mask(Role::Vo))),
        AddDamageByNumOfSp       => Damage(Cardinal(effect_value as f64, role_mask(Role::Sp))),
        AddDamageByNumOfSk       => Damage(Cardinal(effect_value as f64, role_mask(Role::Sk))),
        AddDamageByNumOfGd       => Damage(Cardinal(effect_value as f64, role_mask(Role::Gd))),

        AddVoltage               => VoPlus(Constant(effect_value as f64)),
        AddVoltageByAppeal       => VoPlus(CardAppeal(pct(effect_value))),
        AddVoltageByStamina      => VoPlus(CardStamina(pct(effect_value))),
        AddVoltageByTechnique    => VoPlus(CardTechnique(pct(effect_value))),

        AddAppealBuff            => match sd.calc_type {
            1 => AppealUpAdd(effect_value as f64, Duration::from(sd)),
            2 => AppealUp(pct(effect_value), Duration::from(sd)),
            3 => AppealUpEx(pct(effect_value), Duration::from(sd)),
            _ => Unimplemented,
        },
        AddVoltageBuff           => TapVoUp(pct(effect_value), Duration::from(sd)),

        SkillEffectType::RemoveShield => SkillEff::RemoveShield(Constant(effect_value as f64)),
        _ => Unimplemented,
    }
}

fn attribute_mask(att: Attribute) -> u64 {
    use Attribute::*;
    match att {
        Smile   => 1 << 16,
        Pure    => 1 << 17,
        Cool    => 1 << 18,
        Active  => 1 << 19,
        Natural => 1 << 20,
        Elegant => 1 << 21,
        Neutral => 1 << 22,
    }
}

fn role_mask(ro: Role) -> u64 {
    use Role::*;
    match ro {
        Vo => 1 << 24,
        Sp => 1 << 25,
        Gd => 1 << 26,
        Sk => 1 << 27,
    }
}

fn member_mask(member: Idol) -> u64 {
    1u64 << (32 + member_ofs(member))
}

fn school_mask(sch: u8) -> u64 {
    1u64 << (32 + sch * 9)
}

const FIRST_YEARS: u64 = 0x041a_c0b0 << 32;
const SECOND_YEARS: u64 = 0x0144_260d << 32;
const THIRD_YEARS: u64 = 0x02a1_1942 << 32;
fn year_mask_m(member: Idol) -> u64 {
    use Idol::*;
    match member {
        Honoka   => SECOND_YEARS,
        Eli      => THIRD_YEARS,
        Kotori   => SECOND_YEARS,
        Umi      => SECOND_YEARS,
        Rin      => FIRST_YEARS,
        Maki     => FIRST_YEARS,
        Nozomi   => THIRD_YEARS,
        Hanayo   => FIRST_YEARS,
        Nico     => THIRD_YEARS,
        Chika    => SECOND_YEARS,
        Riko     => SECOND_YEARS,
        Kanan    => THIRD_YEARS,
        Dia      => THIRD_YEARS,
        You      => SECOND_YEARS,
        Yohane   => FIRST_YEARS,
        Hanamaru => FIRST_YEARS,
        Mari     => THIRD_YEARS,
        Ruby     => FIRST_YEARS,
        Ayumu    => SECOND_YEARS,
        Kasumi   => FIRST_YEARS,
        Shizuku  => FIRST_YEARS,
        Karin    => THIRD_YEARS,
        Ai       => SECOND_YEARS,
        Kanata   => THIRD_YEARS,
        Setsuna  => SECOND_YEARS,
        Emma     => THIRD_YEARS,
        Rina     => FIRST_YEARS,
        Shioriko => FIRST_YEARS,
    }
}

const PRINTEMPS: u64   = 0b010_000_101 << 32;
const BIBI: u64        = 0b100_010_010 << 32;
const LILY_WHITE: u64  = 0b001_101_000 << 32;

const CYARON: u64      = 0b100_010_001 << (32 + 9);
const GUILTY_KISS: u64 = 0b010_100_010 << (32 + 9);
const AZALEA: u64      = 0b001_001_100 << (32 + 9);

const AZUNA: u64       = 0b001_000_101 << (32 + 18);
const QU4RTZ: u64      = 0b110_100_010 << (32 + 18);
const DIVER_DIVA: u64  = 0b000_011_000 << (32 + 18);
fn subunit_mask_m(member: Idol) -> u64 {
    use Idol::*;
    match member {
        Honoka   => PRINTEMPS,
        Eli      => BIBI,
        Kotori   => PRINTEMPS,
        Umi      => LILY_WHITE,
        Rin      => LILY_WHITE,
        Maki     => BIBI,
        Nozomi   => LILY_WHITE,
        Hanayo   => PRINTEMPS,
        Nico     => BIBI,
        Chika    => CYARON,
        Riko     => GUILTY_KISS,
        Kanan    => AZALEA,
        Dia      => AZALEA,
        You      => CYARON,
        Yohane   => GUILTY_KISS,
        Hanamaru => AZALEA,
        Mari     => GUILTY_KISS,
        Ruby     => CYARON,
        Ayumu    => AZUNA,
        Kasumi   => QU4RTZ,
        Shizuku  => AZUNA,
        Karin    => DIVER_DIVA,
        Ai       => DIVER_DIVA,
        Kanata   => QU4RTZ,
        Setsuna  => AZUNA,
        Emma     => QU4RTZ,
        Rina     => QU4RTZ,
        Shioriko => 0,
    }
}

fn member_ofs(member: Idol) -> u8 {
    use Idol::*;
    match member {
        Honoka   => 0,
        Eli      => 1,
        Kotori   => 2,
        Umi      => 3,
        Rin      => 4,
        Maki     => 5,
        Nozomi   => 6,
        Hanayo   => 7,
        Nico     => 8,
        Chika    => 9,
        Riko     => 10,
        Kanan    => 11,
        Dia      => 12,
        You      => 13,
        Yohane   => 14,
        Hanamaru => 15,
        Mari     => 16,
        Ruby     => 17,
        Ayumu    => 18,
        Kasumi   => 19,
        Shizuku  => 20,
        Karin    => 21,
        Ai       => 22,
        Kanata   => 23,
        Setsuna  => 24,
        Emma     => 25,
        Rina     => 26,
        Shioriko => 27,
    }
}

