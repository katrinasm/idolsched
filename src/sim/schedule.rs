use crate::state::SearchState;
use super::PlayGlob;
use super::acc_handle::AccHandle;
use rand::Rng;
use rand::distributions::Uniform;
use rand::seq::SliceRandom;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Schedule {
    pub cards: [usize; 9], // album indexes; RRRGGGBBB, same as acc screen in-game
    pub sp3: [usize; 3], // indexes into `cards`; center, left, right
    pub accs: [AccHandle; 9], // inventory indexes; RRRGGGBBB
}

impl Default for Schedule {
    fn default() -> Schedule {
        Schedule {
            cards: [0, 1, 2, 3, 4, 5, 6, 7, 8],
            sp3: [0, 1, 2],
            accs: [AccHandle::empty(); 9],
        }
    }
}

impl Schedule {
    pub fn new_random<R: Rng + ?Sized>(rng: &mut R, album_size: usize, inv_size: usize) -> Schedule {
        assert!(album_size >= 9);
        assert!(inv_size < 0x8000);
        let mut cards = [0; 9];
        let sp3 = [0, 1, 2];
        let mut accs = [AccHandle::empty(); 9];
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
            accs[i] = AccHandle::from(v as u16);
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
    cards: [usize; 9],
    accs: [AccHandle; 9],
    sp3: [usize; 3],
    sp_done: bool,
    cards_done: bool,
    idx: usize,
    green_done: bool,
    green_idx: usize,
    album_size: usize,
    inv_size: u16,
    saved_card: usize,
    saved_acc: AccHandle,
    new_index: bool,
}

impl Iterator for ScheduleIterator {
    type Item = Schedule;
    fn size_hint(&self) -> (usize, Option<usize>) {
        let green_swaps = 3 * 6; // 3 cards in green, 6 to swap them with
        let nonteam_cards = self.album_size - 9;
        let card_replacements = 9 * nonteam_cards;
        let team_accs = self.accs.iter().filter(|han| !han.is_empty()).count();
        let nonteam_accs = self.inv_size as usize - team_accs;
        let acc_replacements = 9 * nonteam_accs;
        let acc_removals = team_accs;
        let succs = green_swaps + card_replacements + acc_replacements + acc_removals;
        // idk where this 2 comes from
        (0, Some(succs + 2))
    }

    fn next(&mut self) -> Option<Schedule> {
        if !self.sp_done {
            // we currently don't do anything with SP so just set up an invariant
            self.saved_card = self.cards[0];
            self.saved_acc = self.accs[0];
            self.sp_done = true;
            self.idx = 3;
        }

        // swap cards into green
        if !self.green_done {
            if self.idx < 9 {
                if self.idx - 1 >= 3 {
                    self.cards[self.idx - 1] = self.cards[self.green_idx];
                }
                self.cards[self.green_idx] = self.cards[self.idx];
                self.cards[self.idx] = self.saved_card;
                self.idx += 1;
                return Some(Schedule { cards: self.cards, sp3: self.sp3, accs: self.accs });
            } else {
                self.cards[self.idx - 1] = self.cards[self.green_idx];
                self.cards[self.green_idx] = self.saved_card;
                self.green_idx += 1;
                if self.green_idx < 3 {
                    self.idx = 3;
                    self.saved_card = self.cards[self.green_idx];
                    return Some(Schedule { cards: self.cards, sp3: self.sp3, accs: self.accs });
                } else {
                    self.idx = 0;
                    self.new_index = true;
                    self.saved_card = self.cards[0];
                    self.green_done = true;
                }
            }
        }

        if !self.cards_done {
            // try new cards
            let mut next_card = if self.new_index {
                self.new_index = false;
                0
            } else {
                self.cards[self.idx] + 1
            };
            self.cards[self.idx] = self.saved_card;

            let mut i = 0;
            while i < 9 {
                if next_card == self.album_size {
                    self.idx += 1;
                    self.new_index = true;
                    if self.idx < 9 {
                        self.saved_card = self.cards[self.idx];
                        return self.next();
                    } else {
                        self.cards_done = true;
                        self.idx = 0;
                        return self.next();
                    }
                } else if self.cards[i] == next_card {
                    next_card += 1;
                    i = 0;
                } else {
                    i += 1;
                }
            }

            self.cards[self.idx] = next_card;
            Some(Schedule { cards: self.cards, sp3: self.sp3, accs: self.accs })
        } else {
            let mut acc_han = if self.new_index {
                self.new_index = false;
                AccHandle::empty()
            } else if let Some(new_han) = self.accs[self.idx].next_max(self.inv_size) {
                new_han
            } else if self.idx + 1 < 9 {
                self.accs[self.idx] = self.saved_acc;
                self.idx += 1;
                self.saved_acc = self.accs[self.idx];
                self.new_index = true;
                return self.next();
            } else {
                return None;
            };
            self.accs[self.idx] = self.saved_acc;

            let mut i = 0;
            while i < 9 {
                if !acc_han.is_empty() && self.accs[i] == acc_han {
                    if let Some(new_han) = acc_han.next_max(self.inv_size) {
                        acc_han = new_han;
                        i = 0;
                    } else {
                        self.idx += 1;
                        self.new_index = true;
                        if self.idx < 9 {
                            self.saved_acc = self.accs[self.idx];
                            return self.next();
                        } else {
                            return None;
                        }
                    }
                } else {
                    i += 1;
                }
            }

            self.accs[self.idx] = acc_han;
            Some(Schedule { cards: self.cards, sp3: self.sp3, accs: self.accs })
        }
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
            cards: self.cards.clone(),
            accs: self.accs.clone(),
            sp3: self.sp3,
            idx: 0,
            album_size: glob.album.len(),
            inv_size: glob.inventory.len().min(0x8000) as u16,
            saved_card: 0,
            saved_acc: AccHandle::empty(),
            sp_done: false,
            cards_done: false,
            green_done: false,
            green_idx: 0,
            new_index: false,
        }
    }
}
