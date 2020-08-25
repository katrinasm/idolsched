use std::collections::HashMap as Map;
use std::hash::Hash;
use crate::state::SearchState;
use rand::Rng;
use rand::seq::SliceRandom;
use std::io::Write;

pub fn anneal<St: SearchState + Hash + std::fmt::Debug, R: Rng + ?Sized>(rng: &mut R, origin: &St, glob: &St::Glob, steps: u32, t0: f64) -> (f64, St, f64) {
    let k_max = steps as f64;
    let mut k = 0.0;
    let mut s = origin.clone();
    let mut se = s.energy(glob);
    let (mut best, mut best_e) = (s.clone(), se);
    let mut previous_values: Map<St, f64> = Map::new();
    let mut cache_hits = 0;
    let mut cache_tries = 0;
    let one_percent = steps / 100;
    let mut countdown = 0;
    let mut percent = 0;

    previous_values.insert(s.clone(), se);

    while k < k_max {
        if countdown == 0 {
            print!("{:>3}%\r", percent);
            std::io::stdout().flush().unwrap();
            countdown = one_percent;
            percent += 1;
        }
        countdown = countdown.saturating_sub(1);

        k += 1.0;
        let temp = t0 * temperature(k / k_max);

        if se < best_e {
            best = s.clone();
            best_e = se;
        }

        let t = random_succ(rng, &s, glob);

        cache_tries += 1;
        let te = if let Some(known) = previous_values.get(&t) {
            cache_hits += 1;
            *known
        } else {
            let new = t.energy(glob);
            previous_values.insert(t.clone(), new);
            new
        };

        let pt = p(se, te, temp).min(1.0);
        if rng.gen_bool(pt) {
            s = t;
            se = te;
        }
    }

    println!("state cache: {} entries, {:.3}% hit rate",
        previous_values.len(),
        (cache_hits as f64 / cache_tries as f64) * 100.0
    );

    (k, best, best_e)
}

fn random_succ<St: SearchState + Hash + std::fmt::Debug, R: Rng + ?Sized>(rng: &mut R, s: &St, glob: &St::Glob) -> St {
    let mut it = s.successors(glob);
    if let (_, Some(upper_bound)) = it.size_hint() {
        loop {
            let n: usize = rng.gen_range(0, upper_bound);
            if let Some(t) = it.skip(n).next() {
                return t;
            }
            it = s.successors(glob);
        }
    } else {
        let mut succs: Vec<St> = it.collect();
        succs.shuffle(rng);
        succs.truncate(1);
        succs.pop().unwrap()
    }
}

// this is a separate function mainly to make it easier to toy with the cooling schedule,
// rather than because it is complicated or anything
fn temperature(r: f64) -> f64 {
    let nr = 1.0 - r;
    nr.sqrt()
}

fn p(se: f64, te: f64, temp: f64) -> f64 {
    if te < se {
        1.0
    } else {
        // te >= se
        // se <= te
        // ==> se - te <= 0.0
        ((se - te) / temp).exp().max(0.0).min(1.0)
    }
}

