use local_search::SearchState;
use super::PlayGlob;
use super::live_show::Status;
use rand::Rng;
use rand::distributions::Uniform;
use rand::seq::SliceRandom;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Schedule {
    pub cards: [usize; 9], // album indexes; GGGBBBRRR
    pub sp3: [usize; 3], // indexes into `cards`; center, left, right
    pub accs: [usize; 9], // inventory indexes; GGGBBBRRR
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
        let sp3 = [1, 0, 2];
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScheduleIterator {
    album_size: usize,
    inv_size: usize,
    cards: [usize; 9],
    accs: [usize; 9],
    sp3: [usize; 3],
    step: usize,
}

impl ScheduleIterator {
    pub fn from_schedule(sched: &Schedule, album_size: usize, inv_size: usize)
    -> ScheduleIterator {
        ScheduleIterator {
            album_size,
            inv_size,
            cards: sched.cards.clone(),
            accs: sched.accs.clone(),
            sp3: sched.sp3.clone(),
            step: 0,
        }
    }
}

impl Default for ScheduleIterator {
    fn default() -> ScheduleIterator {
        ScheduleIterator {
            album_size: 50,
            inv_size: 20,
            cards: [0,1,2,3,4,5,6,7,8],
            accs: [0,1,2,3,4,5,6,7,8],
            sp3: [0,1,2],
            step: 0,
        }
    }
}

impl Iterator for ScheduleIterator {
    type Item = Schedule;
    fn size_hint(&self) -> (usize, Option<usize>) {
        let green_swaps = 3 * 6; // 3 cards in green, 6 to swap them with
        let nonteam_cards = self.album_size - 9;
        let card_replacements = 9 * nonteam_cards;
        let nonteam_accs = self.inv_size - 9;
        let acc_replacements = 9 * nonteam_accs;
        let possible_succs = green_swaps + card_replacements + acc_replacements;
        (0, Some(possible_succs - self.step))
    }

    fn next(&mut self) -> Option<Schedule> {
        use std::mem::swap;

        let mut out_sched = Schedule { sp3: self.sp3, accs: self.accs, cards: self.cards };

        let green_swaps = 3 * 6; // 3 cards in green, 6 to swap them with
        let nonteam_cards = self.album_size - 9;
        let card_replacements = 9 * nonteam_cards;
        let nonteam_accs = self.inv_size - 9;
        let acc_replacements = 9 * nonteam_accs;

        let last_green = green_swaps;
        let last_card = last_green + card_replacements;
        let last_acc = last_card + acc_replacements;

        if self.step < last_green {
            let slot_green = self.step / 6;
            let slot_other = self.step % 6;

            let (green_strat, other_strats) = out_sched.cards.split_at_mut(3);
            swap(&mut green_strat[slot_green], &mut other_strats[slot_other]);

            self.step += 1;
            return Some(out_sched);
        } else if self.step < last_card {
            let substep = self.step - last_green;
            let slot = substep / nonteam_cards;
            let candidates_needed = substep % nonteam_cards + 1;
            let mut cand_i = 0;
            let mut valid_cands = 0;
            while valid_cands < candidates_needed {
                if self.cards.iter().any(|curr_i| *curr_i == cand_i) {
                    cand_i += 1;
                } else {
                    valid_cands += 1;
                    if valid_cands < candidates_needed {
                        cand_i += 1;
                    }
                }
            }
            out_sched.cards[slot] = cand_i;
            self.step += 1;
            return Some(out_sched);
        } else if self.step < last_acc {
            let substep = self.step - last_card;
            let slot = substep / nonteam_accs;
            let candidates_needed = substep % nonteam_accs + 1;
            let mut cand_i = 0;
            let mut valid_cands = 0;
            while valid_cands < candidates_needed {
                if self.accs.iter().any(|curr_i| *curr_i == cand_i) {
                    cand_i += 1;
                } else {
                    valid_cands += 1;
                    if valid_cands < candidates_needed {
                        cand_i += 1;
                    }
                }
            }
            out_sched.accs[slot] = cand_i;
            self.step += 1;
            return Some(out_sched);
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Schedule> {
        self.step += n;
        self.next()
    }
}

impl SearchState for Schedule {
    type Glob = PlayGlob;
    type Buf = Status;
    type Iter = ScheduleIterator;
    fn energy(&self, glob: &PlayGlob, buf: &mut Status) -> f64 {
        -glob.est_voltage(self, buf)
    }

    fn successors(&self, glob: &PlayGlob) -> ScheduleIterator {
        ScheduleIterator::from_schedule(self, glob.album.len(), glob.inventory.len())
    }
}
