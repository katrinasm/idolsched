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

