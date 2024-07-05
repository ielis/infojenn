use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;

use ontolius::prelude::*;
use thiserror::Error;

use crate::feature::{FrequencyAware, Observable, ObservationState};
use crate::item::AnnotatedItem;

#[derive(Debug, Error)]
pub enum IcCalculationError {
    #[error("{0}")]
    OntologyError(Cow<'static, str>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TermIC {
    pub present: f64,
    pub excluded: f64,
}

pub trait IcContainer {
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

impl<'a> IcContainer for HashMap<&'a TermId, TermIC> {
    fn get_term_ic(&self, id: &TermId) -> Option<&TermIC> {
        self.get(id)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl<'a> IcContainer for BTreeMap<&'a TermId, TermIC> {
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

    fn compute_ic<I, A>(&self, items: I) -> Result<Self::Container, IcCalculationError>
    where
        A: AnnotatedItem,
        I: IntoIterator<Item = A>;
}

pub struct CohortIcCalculator<'o, O> {
    hpo: &'o O,
    module_root: &'o TermId,
}

impl<'o, O> CohortIcCalculator<'o, O> {
    pub fn new(hpo: &'o O, module_root: &'o TermId) -> Self {
        CohortIcCalculator { hpo, module_root }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct TermCount {
    present: u32,
    exluded: u32,
}

impl<'o, OI, O> IcCalculator for CohortIcCalculator<'o, O>
where
    OI: TermIdx + HierarchyIdx + Hash,
    O: Ontology<Idx = OI>,
{
    type Container = HashMap<&'o TermId, TermIC>;

    fn compute_ic<I, A>(&self, items: I) -> Result<HashMap<&'o TermId, TermIC>, IcCalculationError>
    where
        A: AnnotatedItem,
        I: IntoIterator<Item = A>,
    {
        let module_idx = self.hpo.id_to_idx(self.module_root);
        if module_idx.is_none() {
            return Err(IcCalculationError::OntologyError(
                format!("Module root {} not in HPO", &self.module_root).into(),
            ));
        }
        let module_idx = module_idx.unwrap();

        let module_term_ids: HashSet<_> = self
            .hpo
            .hierarchy()
            .descendants_of(module_idx)
            .chain(std::iter::once(&module_idx))
            .collect();

        let mut idx2count: HashMap<OI, TermCount> = HashMap::with_capacity(module_term_ids.len());

        for item in items.into_iter() {
            for annotation in item.annotations() {
                if let Some(idx) = self.hpo.id_to_idx(annotation.identifier()) {
                    if module_term_ids.contains(&idx) {
                        match annotation.observation_state() {
                            ObservationState::Present => {
                                idx2count.entry(idx).or_default().present += annotation.numerator();
                                for anc in self.hpo.hierarchy().ancestors_of(idx) {
                                    if module_term_ids.contains(anc) {
                                        idx2count.entry(*anc).or_default().present +=
                                            annotation.numerator();
                                    }
                                }
                            }
                            ObservationState::Excluded => {
                                idx2count.entry(idx).or_default().exluded += annotation.numerator();
                                for desc in self.hpo.hierarchy().descendants_of(idx) {
                                    /*
                                      Unlike in `ObservationState::Present` arm, we do not need
                                      to check if `desc` is contained in `module_term_ids`,
                                      since Ontology DAG guarantees this for any `idx`
                                      contained in `module_term_ids`.
                                    */
                                    idx2count.entry(*desc).or_default().exluded +=
                                        annotation.numerator();
                                }
                            }
                        }
                    }
                } else {
                    return Err(IcCalculationError::OntologyError(
                        format!("Annotation ID {} not in HPO", annotation.identifier()).into(),
                    ));
                }
            }
        }

        if idx2count.is_empty() {
            return Ok(HashMap::new());
        }

        let pop_present_count = idx2count[&module_idx].present as f64;

        /* 
        We use max of the *entire* excluded count set, 
        as opposed to just taking the max of the descendants of a `term_id` in question.
        */
        let pop_excluded_count = idx2count
            .values()
            .max_by_key(|&count| count.exluded)
            .map(|count| count.exluded)
            // We only get here if `idx2count`` is not empty.
            .expect("Idx2count should not be empty") as f64;

        let term_id2ic: HashMap<&TermId, TermIC> = idx2count
            .iter()
            .map(|(idx, count)| {
                let term_id = self
                    .hpo
                    .idx_to_term_id(*idx)
                    .expect("Index was obtained from ontology so it should be there");
                let present_ic = f64::log2(pop_present_count / count.present as f64);
                let excluded_ic = f64::log2(pop_excluded_count / count.exluded as f64);
                (
                    term_id,
                    TermIC {
                        present: present_ic,
                        excluded: excluded_ic,
                    },
                )
            })
            .collect();

        Ok(term_id2ic)
    }
}
