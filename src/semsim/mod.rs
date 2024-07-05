use ontolius::prelude::*;

use crate::{feature::Observable, item::AnnotatedItem};
use anyhow::Result;

pub mod ic;

pub trait ObservableFeature: Identified + Observable {}

impl<T> ObservableFeature for T where T: Identified + Observable {}

pub trait SimilarityMeasure<F>
where
    F: ObservableFeature,
{
    fn compute(&self, left: &F, right: &F) -> Result<f64>;
}

pub trait SimilarityMeasureFactory<F>
where
    F: ObservableFeature,
{
    type Measure: SimilarityMeasure<F>;

    fn create_measure<T>(&self, items: &[T]) -> Result<Self::Measure>
    where
        T: AnnotatedItem<Annotation = F>;
}
