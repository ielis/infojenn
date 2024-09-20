use anyhow::Result;

pub mod ic;

pub trait SimilarityMeasure<F> {
    fn compute(&self, left: &F, right: &F) -> Result<f64>;
}
