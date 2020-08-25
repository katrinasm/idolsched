pub mod error;
pub mod sim;
mod cards_api;
mod anneal;
mod state;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use sim::accessory::Acc;
use sim::card::Card;

pub fn solve(album: Vec<Card>, inventory: Vec<Acc>, step_count: u32) -> (sim::schedule::Schedule, f64) {
    let song = sim::song::TEST_SONG.clone();
    let glob = sim::PlayGlob { album, inventory, song };
    let mut rng = SmallRng::from_entropy();
    let s0 = sim::schedule::Schedule::new_random(&mut rng, glob.album.len(), glob.inventory.len());
    let (_steps, final_sched, energy) = anneal::anneal(&mut rng, &s0, &glob, step_count, 10_000.0);
    (final_sched, -energy)
}

