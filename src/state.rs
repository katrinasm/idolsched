pub trait SearchState: Clone + Eq {
    type Glob;
    type Iter: Iterator<Item=Self>;
    fn energy(&self, glob: &<Self as SearchState>::Glob) -> f64;
    fn successors(&self, glob: &<Self as SearchState>::Glob) -> <Self as SearchState>::Iter;
}

