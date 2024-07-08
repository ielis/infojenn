use std::collections::{BTreeMap, HashMap};

use ontolius::prelude::*;

use crate::item::AnnotatedItem;

pub mod cohort;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TermIC {
    pub present: f64,
    pub excluded: f64,
}

pub trait IcContainer {

    fn iter_term_ids(&self) -> impl Iterator<Item = &TermId>;

    fn get_term_ic(&self, id: &TermId) -> Option<&TermIC>;

    fn get_present_term_ic(&self, id: &TermId) -> Option<&f64> {
        self.get_term_ic(id).map(|x| &x.present)
    }

    fn get_excluded_term_ic(&self, id: &TermId) -> Option<&f64> {
        self.get_term_ic(id).map(|x| &x.excluded)
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IcContainer for HashMap<TermId, TermIC> {

    fn iter_term_ids(&self) -> impl Iterator<Item = &TermId> {
        self.keys()
    }

    fn get_term_ic(&self, id: &TermId) -> Option<&TermIC> {
        self.get(id)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl IcContainer for BTreeMap<TermId, TermIC> {

    fn iter_term_ids(&self) -> impl Iterator<Item = &TermId> {
        self.keys()
    }

    fn get_term_ic(&self, id: &TermId) -> Option<&TermIC> {
        self.get(id)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// Not object-safe due to generics in `compute_ic`.
pub trait IcCalculator {
    type Container: IcContainer;

    fn compute_ic<A>(&self, items: &[A]) -> anyhow::Result<Self::Container>
    where
        A: AnnotatedItem;
}
