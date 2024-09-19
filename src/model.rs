use ontolius::base::Identified;

use std::ops::Div;

use ontolius::base::TermId;

/// An enum to represent if a feature was present or excluded in the study subject(s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObservationState {
    /// The feature was present.
    Present,
    /// Presence of a feature was explicitly ruled out by an investigation.
    Excluded,
}

/// `Observable` entity is either in a *present* or an *excluded* state
/// in the investigated item.
///
/// For instance, a phenotypic feature such as [Polydactyly](https://hpo.jax.org/browse/term/HP:0010442)
/// can either be present or excluded in the study subject.
pub trait Observable {
    /// Get the observation state of a feature
    fn observation_state(&self) -> ObservationState;

    /// Test if the feature was observed in one or more items.
    fn is_present(&self) -> bool {
        match self.observation_state() {
            ObservationState::Present => true,
            ObservationState::Excluded => false,
        }
    }

    /// Test if the feature was not observed in any of the items.
    fn is_excluded(&self) -> bool {
        match self.observation_state() {
            ObservationState::Present => false,
            ObservationState::Excluded => true,
        }
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
    fn observation_state(&self) -> ObservationState {
        match self.numerator() {
            0 => ObservationState::Excluded,
            _ => ObservationState::Present,
        }
    }
}

/// A feature of a subject.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndividualFeature {
    identifier: TermId,
    observation_state: ObservationState,
}

impl IndividualFeature {
    pub fn new(identifier: TermId, is_present: bool) -> Self {
        IndividualFeature {
            identifier,
            observation_state: match is_present {
                true => ObservationState::Present,
                false => ObservationState::Excluded,
            },
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
        1
    }

    fn denominator(&self) -> u32 {
        1
    }
}

/// The aggregated feature represents data for the feature ascertained from a cohort.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

pub trait AnnotatedItem {
    type Annotation: Identified + FrequencyAware;

    fn annotations(&self) -> &[Self::Annotation];

    fn present_annotations(&self) -> impl Iterator<Item = &Self::Annotation> {
        self.annotations().iter().filter(|&a| a.is_present())
    }

    fn excluded_annotations(&self) -> impl Iterator<Item = &Self::Annotation> {
        self.annotations().iter().filter(|&a| a.is_excluded())
    }
}

impl<'a, T> AnnotatedItem for &'a [T]
where
    T: Identified + FrequencyAware,
{
    type Annotation = T;

    fn annotations(&self) -> &[Self::Annotation] {
        self
    }
}

impl<T> AnnotatedItem for Vec<T>
where
    T: Identified + FrequencyAware,
{
    type Annotation = T;

    fn annotations(&self) -> &[Self::Annotation] {
        self
    }
}

pub trait Cohort {
    type Member: AnnotatedItem;

    fn members(&self) -> &[Self::Member];
}

impl<'a, T> Cohort for &'a [T]
where
    T: AnnotatedItem,
{
    type Member = T;

    fn members(&self) -> &[Self::Member] {
        self
    }
}

impl<T> Cohort for Vec<T>
where
    T: AnnotatedItem,
{
    type Member = T;

    fn members(&self) -> &[Self::Member] {
        self
    }
}
