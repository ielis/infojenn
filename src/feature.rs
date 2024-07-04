use std::ops::Div;

use ontolius::base::{Identified, TermId};

/// `Observable` entity is either in a *present* or an *excluded* state
/// in the investigated item.
///
/// For instance, a phenotypic feature such as [Polydactyly](https://hpo.jax.org/browse/term/HP:0010442)
/// can either be present or excluded in the study subject.
pub trait Observable {
    /// Test if the feature was observed in one or more items.
    fn is_present(&self) -> bool;

    /// Test if the feature was not observed in any of the items.
    fn is_excluded(&self) -> bool {
        !self.is_present()
    }
}

/// `FrequencyAware` entity describes the frequency of a feature in one or more annotated items.
///
/// For instance, we can represent the feature frequency in a collection of items, such as presence of
/// a phenotypic feature, such as [Polydactyly](https://hpo.jax.org/browse/term/HP:0010442),
/// in the cohort.
///
/// The absolute counts are accessible via the `numerator` and `denominator` attributes.
///
/// **IMPORTANT**: the implementor must ensure that the `denominator` is a *positive* `u32`.
///
/// All `FrequenceyAware` types also implement [`Observable`].
pub trait FrequencyAware {
    /// Get the numerator, representing the count of annotated items where the feature was present.
    fn numerator(&self) -> u32;

    /// Get the denominator, representing the total count of annotated items investigated
    /// for presence/absence of the feature.
    fn denominator(&self) -> u32;

    /// Get the fraction of the feature in the annotated item(s) as `f64`.
    fn frequency(&self) -> f64 {
        f64::div(self.numerator() as f64, self.denominator() as f64)
    }
}

impl<T> Observable for T
where
    T: FrequencyAware,
{
    fn is_present(&self) -> bool {
        self.numerator() > 0
    }
}

pub trait Feature: Identified + FrequencyAware {}

/// A feature of a subject.
pub struct IndividualFeature {
    identifier: TermId,
    is_present: bool,
}

impl IndividualFeature {
    pub fn new(identifier: TermId, is_present: bool) -> Self {
        IndividualFeature {
            identifier,
            is_present,
        }
    }
}

impl Identified for IndividualFeature {
    fn identifier(&self) -> &TermId {
        &self.identifier
    }
}

impl FrequencyAware for IndividualFeature {
    fn numerator(&self) -> u32 {
        match self.is_present {
            true => 1,
            false => 0,
        }
    }

    fn denominator(&self) -> u32 {
        1
    }
}

impl Feature for IndividualFeature {}

/// The aggregated feature represents data for the feature ascertained from a cohort.
pub struct AggregatedFeature {
    identifier: TermId,
    numerator: u32,
    denominator: u32,
}

impl AggregatedFeature {
    pub fn new(identifier: TermId, numerator: u32, denominator: u32) -> Self {
        AggregatedFeature {
            identifier,
            numerator,
            denominator,
        }
    }
}

impl Identified for AggregatedFeature {
    fn identifier(&self) -> &TermId {
        &self.identifier
    }
}

impl FrequencyAware for AggregatedFeature {
    fn numerator(&self) -> u32 {
        self.numerator
    }

    fn denominator(&self) -> u32 {
        self.denominator
    }
}

impl Feature for AggregatedFeature {}
