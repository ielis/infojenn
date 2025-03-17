use std::collections::{BTreeMap, HashMap};

use ontolius::TermId;

use crate::model::ObservationState;

pub mod cohort;

pub trait IcContainer {
    fn iter_term_ids(&self) -> impl Iterator<Item = &TermId>;

    fn get_term_ic(&self, id: &TermId, state: ObservationState) -> Option<f64>;

    fn get_present_term_ic(&self, id: &TermId) -> Option<f64> {
        self.get_term_ic(id, ObservationState::Present)
    }

    fn get_excluded_term_ic(&self, id: &TermId) -> Option<f64> {
        self.get_term_ic(id, ObservationState::Excluded)
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TermIC {
    pub present: f64,
    pub excluded: f64,
}

impl IcContainer for HashMap<TermId, TermIC> {
    fn iter_term_ids(&self) -> impl Iterator<Item = &TermId> {
        self.keys()
    }

    fn get_term_ic(&self, id: &TermId, state: ObservationState) -> Option<f64> {
        self.get(id).map(|term_ic| match state {
            ObservationState::Present => term_ic.present,
            ObservationState::Excluded => term_ic.excluded,
        })
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl IcContainer for BTreeMap<TermId, TermIC> {
    fn iter_term_ids(&self) -> impl Iterator<Item = &TermId> {
        self.keys()
    }

    fn get_term_ic(&self, id: &TermId, state: ObservationState) -> Option<f64> {
        self.get(id).map(|term_ic| match state {
            ObservationState::Present => term_ic.present,
            ObservationState::Excluded => term_ic.excluded,
        })
    }

    fn len(&self) -> usize {
        self.len()
    }
}

pub trait IcCalculator<C> {
    type Container: IcContainer;

    fn compute_ic(&self, cohort: &C) -> anyhow::Result<Self::Container>;
}
