use super::pct;
use crate::mapdb::Song;
use super::card::Card;
use super::schedule::Schedule;
use super::accessory::Acc;
use super::basic_data::*;
use super::skill::{Skill, SkillEff, ValueType, Duration};

#[derive(Debug, Default)]
struct StatList {
    appeal: [f64; 9],
    stamina: [f64; 9],
    technique: [f64; 9],
    crit_rate_bonus: [f64; 9],
    crit_power: [f64; 9],
    skill_mask: [u64; 9],
    att_mod: [f64; 9],
    tap_skill: [Skill; 9],
    cap_tap: [f64; 9],
    cap_skill: [f64; 9],
    mod_vo: [f64; 3],
    mod_sp: [f64; 3],
    mod_gd: [f64; 3],
    mod_sk: [f64; 3],
    max_stam: f64,
}

#[derive(Debug)]
struct Status {
    note_pos: usize,
    note_cnt: usize,
    stam: f64,
    voltage: f64,
    shield: f64,
    strat: usize,
    buff_appeal_add: Vec<[f64; 9]>,
    buff_appeal: Vec<[f64; 9]>,
    buff_appeal_ex: Vec<[f64; 9]>,
    buff_tapvo: Vec<[f64; 9]>,
    debuff_appeal: Vec<[f64; 9]>,
}

const TIMING: f64 = 1.1;

pub fn run(song: &Song, album: &Vec<Card>, inventory: &Vec<Acc>, sched: &Schedule) -> f64 {
    let stat_list = make_stat_list(song, album, inventory, sched);

    let dpn = song.note_stamina_reduce as f64;

    let mut status = Status {
        stam: stat_list.max_stam,
        note_pos: 0,
        note_cnt: song.kt_notes,
        voltage: 0.0,
        shield: 0.0,
        strat: 0,
        buff_appeal_add: vec![[0.0; 9]; song.kt_notes as usize],
        buff_appeal: vec![[0.0; 9]; song.kt_notes as usize],
        buff_appeal_ex: vec![[0.0; 9]; song.kt_notes as usize],
        buff_tapvo: vec![[0.0; 9]; song.kt_notes as usize],
        debuff_appeal: vec![[0.0; 9]; song.kt_notes as usize],
    };

    while status.note_pos < status.note_cnt {
        let card_pos = status.strat * 3 + status.note_pos % 3;
        proc_skill(&mut status, &stat_list, &stat_list.tap_skill[card_pos], card_pos);
        let mut volts = appeal(&stat_list, &status, card_pos);
        volts += volts * crit_rate(&stat_list, card_pos) * crit_power(&stat_list, card_pos);
        volts *= TIMING;
        volts *= combo_mod(status.note_pos);
        volts *= 1.0 + status.buff_tapvo[status.note_pos][card_pos];
        volts *= stat_list.mod_vo[status.strat];
        volts *= stat_list.att_mod[card_pos];
        volts *= stam_mod(status.stam, stat_list.max_stam);
        status.voltage += volts.min(stat_list.cap_tap[card_pos]);

        let in_damage = dpn * stat_list.mod_gd[status.strat];
        let (nshield, damage) = ((status.shield - in_damage).max(0.0), (in_damage - status.shield).max(0.0));
        status.shield = nshield;
        status.stam = (status.stam - damage).max(0.0);
        if song.lose_at_death && status.stam == 0.0 {
            // this is a hack,
            // which lets the search find teams that are closer to surviving
            // before it finds one that actually survives
            return status.voltage / 10_000.0;
        }

        status.note_pos += 1;
    }
    status.voltage
}

fn proc_skill(status: &mut Status, stat_list: &StatList, skill: &Skill, card_pos: usize) {
    let p = skill.prob;
    match skill.eff {
        SkillEff::Heal(v) => status.stam =
            stat_list.max_stam.min(status.stam + p * get_val(&stat_list, &status, v, card_pos) * stat_list.att_mod[card_pos]),
        SkillEff::Shield(v) => status.shield =
            stat_list.max_stam.min(status.shield + p * get_val(&stat_list, &status, v, card_pos) * stat_list.att_mod[card_pos]),
        SkillEff::RemoveShield(v) => status.shield =
            (0.0f64).max(status.shield - p * get_val(&stat_list, &status, v, card_pos)),
        SkillEff::VoPlus(v) => status.voltage +=
            stat_list.cap_skill[card_pos]
            .min(stat_list.mod_vo[status.strat] * p * get_val(&stat_list, &status, v, card_pos) * stat_list.att_mod[card_pos]),
        SkillEff::AppealUpAdd(v, dur) => {
            let deltas = make_deltas(stat_list, skill, p * v);
            for_duration(status, dur, |status, i| add9(&mut status.buff_appeal_add[i], &deltas));
        },
        SkillEff::AppealUp(v, dur) => {
            let deltas = make_deltas(stat_list, skill, p * v);
            for_duration(status, dur, |status, i| add9(&mut status.buff_appeal[i], &deltas));
        },
        SkillEff::AppealUpEx(v, dur) => {
            let deltas = make_deltas(stat_list, skill, p * v);
            for_duration(status, dur, |status, i| add9(&mut status.buff_appeal_ex[i], &deltas));
        },
        SkillEff::TapVoUp(v, dur) => {
            let deltas = make_deltas(stat_list, skill, p * v);
            for_duration(status, dur, |status, i| add9(&mut status.buff_tapvo[i], &deltas));
        },
        _ => {},
    }
}

fn make_deltas(stat_list: &StatList, skill: &Skill, delta: f64) -> [f64; 9] {
    let mut deltas = [delta; 9];
    for i in 0 .. 9 {
        if stat_list.skill_mask[i] & skill.target_mask == 0 {
            deltas[i] = 0.0;
        }
    }
    deltas
}

fn add9(dst: &mut [f64; 9], src: &[f64; 9]) {
    for i in 0 .. 9 {
        dst[i] += src[i];
    }
}

fn for_duration<F: Fn(&mut Status, usize)>(status: &mut Status, dur: Duration, f: F) {
    use Duration::*;
    match dur {
        Permanent => for i in status.note_pos .. status.note_cnt { f(status, i) },
        Turn(n) => for i in status.note_pos .. status.note_cnt.min(status.note_pos + n as usize) { f(status, i) },
        _ => {},
    }
}

fn make_stat_list(song: &Song, album: &Vec<Card>, inventory: &Vec<Acc>, sched: &Schedule) -> StatList {
    let mut stat_list = StatList::default();
    let mut mod_appeal = [1.0; 9];
    let mut mod_stamina = [1.0; 9];
    let mut mod_technique = [1.0; 9];
    let mut att = [Attribute::Neutral; 9];
    for pos in 0..3 {
        stat_list.mod_vo[pos] = 1.0;
        stat_list.mod_sp[pos] = 1.0;
        stat_list.mod_gd[pos] = 1.0;
        stat_list.mod_sk[pos] = 1.0;
    }
    for pos in 0..9 {
        stat_list.crit_power[pos] = 0.5;
    }
    for (pos, &card_i) in sched.cards.iter().enumerate() {
        let card = &album[card_i];
        stat_list.appeal[pos] = card.appeal as f64;
        stat_list.stamina[pos] = card.stamina as f64;
        stat_list.technique[pos] = card.technique as f64;
        stat_list.crit_rate_bonus[pos] = if card.crit_rate_bonus { 0.15 } else { 0.0 };
        stat_list.skill_mask[pos] = card.skill_mask | (1 << pos);
        stat_list.tap_skill[pos] = card.tap_skill.imbue_pos(pos);
        stat_list.cap_tap[pos] = song.note_voltage_upper_limit as f64;
        stat_list.cap_skill[pos] = song.skill_voltage_upper_limit as f64;
        att[pos] = card.attribute;
        stat_list.att_mod[pos] = if card.attribute == song.default_attribute {
            1.2
        } else {
            1.0
        };

        let strat_pos = pct(card.role_effect.positive_value);
        let strat_neg = pct(card.role_effect.negative_value);
        let strat = pos / 3;
        match card.role_effect.positive_type {
            Role::Vo => stat_list.mod_vo[strat] += strat_pos,
            Role::Sp => stat_list.mod_sp[strat] += strat_pos,
            Role::Gd => stat_list.mod_gd[strat] -= strat_pos,
            Role::Sk => stat_list.mod_sk[strat] += strat_pos,
        }
        match card.role_effect.negative_type {
            Role::Vo => stat_list.mod_vo[strat] -= strat_neg,
            Role::Sp => stat_list.mod_sp[strat] -= strat_neg,
            Role::Gd => stat_list.mod_gd[strat] += strat_neg,
            Role::Sk => stat_list.mod_sk[strat] -= strat_neg,
        }
    }

    for (pos, &card_i) in sched.cards.iter().enumerate() {
        let card = &album[card_i];
        for pre_sk in card.etc_skills.iter() {
            let sk = pre_sk.imbue_pos(pos);
            match sk.eff {
                SkillEff::AppealPlus(v) => do_passive(&mut mod_appeal, &stat_list.skill_mask, sk.target_mask, v),
                SkillEff::StaminaPlus(v) => do_passive(&mut mod_stamina, &stat_list.skill_mask, sk.target_mask, v),
                SkillEff::TechniquePlus(v) => do_passive(&mut mod_technique, &stat_list.skill_mask, sk.target_mask, v),
                SkillEff::CritRatePlus(v) => do_passive(&mut stat_list.crit_rate_bonus, &stat_list.skill_mask, sk.target_mask, v),
                SkillEff::CritPlus(v) => do_passive(&mut stat_list.crit_power, &stat_list.skill_mask, sk.target_mask, v),
                SkillEff::TypePlus(v) => {
                    let strat = card_i / 3;
                    match card.role_effect.positive_type {
                        Role::Vo => stat_list.mod_vo[strat] += v,
                        Role::Sp => stat_list.mod_sp[strat] += v,
                        Role::Gd => stat_list.mod_gd[strat] -= v,
                        Role::Sk => stat_list.mod_sk[strat] += v,
                    }
                },
                _ => {},
            }
        }
    }

    stat_list.appeal.iter_mut().zip(mod_appeal.iter()).for_each(|(x, r)| *x *= *r);
    stat_list.stamina.iter_mut().zip(mod_stamina.iter()).for_each(|(x, r)| *x *= *r);
    stat_list.technique.iter_mut().zip(mod_technique.iter()).for_each(|(x, r)| *x *= *r);

    for strat_pos in 0..3 {
        let strat_i = 3 * strat_pos;
        let strat_accs = &sched.accs[strat_i .. strat_i + 3];
        for &acc_i in strat_accs.iter() {
            let acc = &inventory[acc_i];
            for card_pos in strat_i .. strat_i + 3 {
                if att[card_pos] == acc.attribute {
                    stat_list.appeal[card_pos] += acc.appeal;
                    stat_list.stamina[card_pos] += acc.stamina;
                    stat_list.technique[card_pos] += acc.technique;
                } else {
                    stat_list.appeal[card_pos] += acc.appeal * 1.1;
                    stat_list.stamina[card_pos] += acc.stamina * 1.1;
                    stat_list.technique[card_pos] += acc.technique * 1.1;
                }
            }
        }
    }

    stat_list.max_stam = stat_list.stamina.iter().sum();

    stat_list
}

fn do_passive(mod_array: &mut [f64; 9], mask_list: &[u64; 9], target_mask: u64, v: f64) {
    for (pos, mask) in mask_list.iter().enumerate() {
        if mask & target_mask != 0 {
            mod_array[pos] += v;
        }
    }
}

fn get_val(stat_list: &StatList, status: &Status, v: ValueType, pos: usize) -> f64 {
    use ValueType::*;
    match v {
        Constant(n) => n,
        CardAppeal(p) => p * appeal(stat_list, status, pos),
        CardStamina(p) => p * stat_list.stamina[pos],
        CardTechnique(p) => p * stat_list.technique[pos],
        _ => 0.0,
    }
}

fn appeal(stat_list: &StatList, status: &Status, pos: usize) -> f64 {
    (stat_list.appeal[pos] + status.buff_appeal_add[status.note_pos][pos])
    * (1.0 + status.buff_appeal[status.note_pos][pos] - status.debuff_appeal[status.note_pos][pos])
    * (1.0 + status.buff_appeal_ex[status.note_pos][pos])
}

fn crit_rate(stat_list: &StatList, pos: usize) -> f64 {
    let m = 1.0 / 34_000.0;
    stat_list.technique[pos] * m + stat_list.crit_rate_bonus[pos]
}

fn crit_power(stat_list: &StatList, pos: usize) -> f64 {
    stat_list.crit_power[pos]
}

fn combo_mod(note_pos: usize) -> f64 {
    let m = 1.0 / 150.0;
    1.0 + 0.05 * (m * note_pos.min(150) as f64)
}

fn stam_mod(stam: f64, max_stam: f64) -> f64 {
    let p = stam / max_stam;
    if p > 0.8 {
        1.0
    } else if p > 0.5 {
        0.8
    } else if p != 0.0 {
        0.6
    } else {
        0.0
    }
}

