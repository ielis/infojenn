use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use itertools::Itertools;
use ontolius::prelude::*;

use crate::{
    feature::{Observable, ObservationState},
    ic::{IcCalculator, IcContainer},
    item::AnnotatedItem,
};
use anyhow::{bail, Result};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{SimilarityMeasure, SimilarityMeasureFactory};

pub struct IcSimilarityMeasureFactory<'o, O, IC> {
    hpo: &'o O,
    ic_calculator: IC,
}

impl<'a, O, IC> IcSimilarityMeasureFactory<'a, O, IC> {
    pub fn new(hpo: &'a O, ic_calculator: IC) -> Self {
        Self { hpo, ic_calculator }
    }
}

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

impl<'o, OI, O, C, IC, T, F> SimilarityMeasureFactory<T, F> for IcSimilarityMeasureFactory<'o, O, IC>
where
    T: AnnotatedItem<Annotation = F>,
    F: Identified + Observable,
    OI: HierarchyIdx + TermIdx + Hash + Sync,
    O: Ontology<Idx = OI> + Sync,
    C: IcContainer + Sync,
    IC: IcCalculator<Container = C>,
{
    type Measure = PrecomputedSimilarityMeasure;

    fn create_measure(&self, items: &[T]) -> Result<Self::Measure>
    {
        let container = self.ic_calculator.compute_ic(items)?;
        let relevant: HashSet<&OI> = container
            .iter_term_ids()
            .map(|t| self.hpo.id_to_idx(t).expect("Should be there!"))
            .collect();

        let combinations: Vec<_> = container.iter_term_ids().combinations(2).collect();

        let tp2ic_mica: HashMap<TermPair, f64> = combinations
            .par_iter()
            .map(|c| {
                let left = c[0];
                let right = c[1];
                let ic_mica = find_ic_mica(left, right, &container, &relevant, self.hpo);
                if ic_mica < 1e-8 {
                    None
                } else {
                    Some((TermPair::new(left.clone(), right.clone()), ic_mica))
                }
            })
            .flatten()
            .collect();

        Ok(PrecomputedSimilarityMeasure { tp2ic_mica })
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

    l_ancestors.intersection(&r_ancestors)
        .map(|&ti| {
            let t = hpo.idx_to_term_id(ti).expect("Should be there");
            ic_container.get_present_term_ic(t).unwrap()
        })
        .max_by(|&l, &r| l.partial_cmp(r).expect("We should not be getting NaNs!"))
        .copied()
        .unwrap_or(0.)
}
