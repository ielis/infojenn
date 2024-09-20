use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use ontolius::prelude::*;

use crate::{
    ic::IcContainer,
    model::{Observable, ObservationState},
};
use anyhow::{bail, Result};

use super::SimilarityMeasure;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TermPair {
    left: TermId,
    right: TermId,
}

impl TermPair {
    pub fn new(left: TermId, right: TermId) -> Self {
        match left.cmp(&right) {
            Ordering::Less | Ordering::Equal => Self { left, right },
            Ordering::Greater => Self {
                left: right,
                right: left,
            },
        }
    }
}

pub struct PrecomputedSimilarityMeasure {
    tp2ic_mica: HashMap<TermPair, f64>,
}

impl<T> SimilarityMeasure<T> for PrecomputedSimilarityMeasure
where
    T: Identified + Observable,
{
    fn compute(&self, left: &T, right: &T) -> Result<f64> {
        match (left.observation_state(), right.observation_state()) {
            (ObservationState::Present, ObservationState::Present) => {
                let tp = TermPair::new(left.identifier().clone(), right.identifier().clone());
                Ok(self.tp2ic_mica.get(&tp).copied().unwrap_or(0.))
            }
            (ObservationState::Present, ObservationState::Excluded) => {
                bail!("Present vs. Excluded is not supported")
            }
            (ObservationState::Excluded, ObservationState::Present) => {
                bail!("Present vs. Excluded is not supported")
            }
            (ObservationState::Excluded, ObservationState::Excluded) => {
                // TODO:
                // Check if they are related and return IC if yes. Alternatively, return `0.``
                todo!()
            }
        }
    }
}

fn find_ic_mica<C, O, OI>(
    left: &TermId,
    right: &TermId,
    ic_container: &C,
    relevant: &HashSet<&OI>,
    hpo: &O,
) -> f64
where
    C: IcContainer,
    OI: HierarchyIdx + TermIdx + Hash,
    O: Ontology<Idx = OI>,
{
    if *left == *right {
        return ic_container.get_present_term_ic(left).copied().unwrap();
    }

    let l_ancestors: HashSet<_> = hpo
        .hierarchy()
        .iter_node_and_ancestors_of(hpo.id_to_idx(left).expect("Should be there"))
        .filter(|&i| relevant.contains(i))
        .collect();

    let r_ancestors: HashSet<_> = hpo
        .hierarchy()
        .iter_node_and_ancestors_of(hpo.id_to_idx(right).expect("Should be there"))
        .filter(|&i| relevant.contains(i))
        .collect();

    l_ancestors
        .intersection(&r_ancestors)
        .map(|&ti| {
            let t = hpo.idx_to_term_id(ti).expect("Should be there");
            ic_container.get_present_term_ic(t).unwrap()
        })
        .max_by(|&l, &r| l.partial_cmp(r).expect("We should not be getting NaNs!"))
        .copied()
        .unwrap_or(0.)
}
