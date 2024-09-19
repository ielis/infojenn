use crate::model::AnnotatedItem;
use anyhow::Result;

pub mod ic;

pub trait SimilarityMeasure<F> {
    fn compute(&self, left: &F, right: &F) -> Result<f64>;
}

pub trait SimilarityMeasureFactory<T, F>
where
    T: AnnotatedItem<Annotation = F>,
{
    type Measure: SimilarityMeasure<F>;

    fn create_measure(&self, items: &[T]) -> Result<Self::Measure>;
}
