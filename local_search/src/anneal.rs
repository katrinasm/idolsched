use crate::{SearchState, ReusableBuffer, SimpleIterSolver};
use rand::Rng;
use rand::seq::SliceRandom;

pub struct Params<R: Rng> {
    pub rng: R,
    pub t0: f64,
    pub alpha: f64,
}

pub struct Annealer<St: SearchState, R: Rng> {
    glob: St::Glob,
    buf: St::Buf,
    rng: R,
    temp: f64,
    alpha: f64,
    last: (St, f64),
}

impl<St: SearchState, R: Rng> SimpleIterSolver<St> for Annealer<St, R> {
    type Pm = Params<R>;
    fn org(origin: St, glob: St::Glob, params: Params<R>) -> Annealer<St, R> {
        let mut buf = St::Buf::create();
        let org_energy = origin.energy(&glob, &mut buf);
        Annealer {
            glob,
            buf,
            rng: params.rng,
            temp: params.t0,
            alpha: params.alpha,
            last: (origin, org_energy)
        }
    }

    fn advance(&mut self) -> Option<(St, f64)> {
        let t = random_succ(&mut self.rng, &self.last.0, &self.glob);
        self.buf.refresh();
        let te = t.energy(&self.glob, &mut self.buf);

        let pt = p(self.last.1, te, self.temp).min(1.0);

        if self.rng.gen_bool(pt) {
            self.last = (t, te);
        }

        self.temp *= self.alpha;

        Some((self.last.0.clone(), self.last.1))
    }
}

fn random_succ<St: SearchState, R: Rng>(rng: &mut R, s: &St, glob: &St::Glob) -> St {
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

