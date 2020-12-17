use crate::state::SearchState;
use super::PlayGlob;
use rand::Rng;
use rand::distributions::Uniform;
use rand::seq::SliceRandom;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Schedule {
    pub cards: [usize; 9], // album indexes; RRRGGGBBB, same as acc screen in-game
    pub sp3: [usize; 3], // indexes into `cards`; center, left, right
    pub accs: [usize; 9], // inventory indexes; RRRGGGBBB
}

impl Default for Schedule {
    fn default() -> Schedule {
        Schedule {
            cards: [0, 1, 2, 3, 4, 5, 6, 7, 8],
            sp3: [0, 1, 2],
            accs: [0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }
}

impl Schedule {
    pub fn new_random<R: Rng + ?Sized>(rng: &mut R, album_size: usize, inv_size: usize) -> Schedule {
        assert!(album_size >= 9);
        assert!(inv_size < 0x8000);
        let mut cards = [0; 9];
        let sp3 = [4, 3, 5];
        let mut accs = [0; 9];
        let card_dist = Uniform::new(0, album_size);
        let acc_dist = Uniform::new(0, inv_size);
        let mut used_flags = Vec::with_capacity(album_size.max(inv_size));

        used_flags.resize(album_size, false);
        for i in 0 .. 9 {
            let mut v = rng.sample(card_dist);
            while used_flags[v] {
                v = rng.sample(card_dist);
            }
            cards[i] = v;
            used_flags[v] = true;
        }

        used_flags.truncate(0);
        used_flags.resize(inv_size, false);
        for i in 0 .. 9 {
            if i == inv_size {
                accs[..].shuffle(rng);
                break;
            }
            let mut v = rng.sample(acc_dist);
            while used_flags[v] {
                v = rng.sample(acc_dist);
            }
            accs[i] = v;
            used_flags[v] = true;
        }

        Schedule { cards, accs, sp3 }
    }
}

// this whole class could definitely be implemented better than it is,
// the current code kind of grew by accretion, but it's not very important
// so it will rot for now
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScheduleIterator {
    album_size: usize,
    inv_size: usize,
    cards: [usize; 9],
    accs: [usize; 9],
    sp3: [usize; 3],
    slot_green: usize,
    slot_cards: usize,
    slot_accs: usize,
    step: usize,
}

impl Iterator for ScheduleIterator {
    type Item = Schedule;
    fn size_hint(&self) -> (usize, Option<usize>) {
        let green_swaps = 3 * 6; // 3 cards in green, 6 to swap them with
        let nonteam_cards = self.album_size - 9;
        let card_replacements = 9 * nonteam_cards;
        let nonteam_accs = self.inv_size - 9;
        let acc_replacements = 9 * nonteam_accs;
        let succs = green_swaps + card_replacements + acc_replacements;
        (0, Some(succs))
    }

    fn next(&mut self) -> Option<Schedule> {
        use std::mem::swap;

        let mut out_sched = Schedule { sp3: self.sp3, accs: self.accs, cards: self.cards };

        while self.slot_green < 3 {
            if self.step < 6 {
                {
                    let (green_strat, other_strats) = out_sched.cards.split_at_mut(3);
                    swap(&mut green_strat[self.slot_green], &mut other_strats[self.step]);
                }
                self.step += 1;
                return Some(out_sched);
            } else {
                self.slot_green += 1;
                self.step = 0;
            }
        }

        while self.slot_cards < 9 {
            if self.step < self.album_size {
                let n = self.album_size.saturating_sub(self.step);
                for i in 0 .. n {
                    let cand_i = self.step + i;
                    if self.cards.iter().all(|curr_i| *curr_i != cand_i) {
                        self.step += 1 + i;
                        out_sched.cards[self.slot_cards] = cand_i;
                        return Some(out_sched);
                    }
                }
            }
            self.slot_cards += 1;
            self.step = 0;
        }

        while self.slot_accs < 9 {
            if self.step < self.inv_size {
                let n = self.inv_size.saturating_sub(self.step);
                for i in 0 .. n {
                    let cand_i = self.step + i;
                    if self.accs.iter().all(|curr_i| *curr_i != cand_i) {
                        self.step += 1 + i;
                        out_sched.accs[self.slot_accs] = cand_i;
                        return Some(out_sched);
                    }
                }
            }
            self.slot_accs += 1;
            self.step = 0;
        }

        None
    }
}

impl SearchState for Schedule {
    type Glob = PlayGlob;
    type Iter = ScheduleIterator;
    fn energy(&self, glob: &PlayGlob) -> f64 {
        -glob.est_voltage(self)
    }

    fn successors(&self, glob: &PlayGlob) -> ScheduleIterator {
        ScheduleIterator {
            album_size: glob.album.len(),
            inv_size: glob.inventory.len(),
            cards: self.cards.clone(),
            accs: self.accs.clone(),
            sp3: self.sp3,
            slot_green: 0,
            slot_cards: 0,
            slot_accs: 0,
            step: 0,
        }
    }
}
