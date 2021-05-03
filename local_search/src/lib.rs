pub mod anneal;

pub trait SearchState: Clone + Eq {
    type Glob;
    type Buf: ReusableBuffer;
    type Iter: Iterator<Item=Self>;
    fn energy(&self, glob: &<Self as SearchState>::Glob, buf: &mut <Self as SearchState>::Buf) -> f64;
    fn successors(&self, glob: &<Self as SearchState>::Glob) -> <Self as SearchState>::Iter;
}

pub trait ReusableBuffer {
    fn create() -> Self;
    fn refresh(&mut self);
}

pub trait SimpleIterSolver<St: SearchState> {
    type Pm;
    fn org(origin: St, glob: St::Glob, params: Self::Pm) -> Self;
    fn advance(&mut self) -> Option<(St, f64)>;
}

pub fn search_n<St: SearchState, Sv: SimpleIterSolver<St>>(solver: &mut Sv, n: u32)
-> Option<(St, f64)> {
    let mut best: Option<(St, f64)> = None;
    let mut i = 0;
    while let Some(new) = solver.advance() {
        if i == n {
            break;
        } else {
            i += 1;
        }
        if let Some(ref v) = best {
            if new.1 < v.1 {
                best = Some(new);
            }
        } else {
            best = Some(new);
        }
    }
    best
}

