use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::model::{Annotated, Observable, ObservationState};

use super::{IcCalculator, TermIC};
use anyhow::Result;
use ontolius::{ontology::HierarchyWalks, Identified, TermId};

pub struct CohortIcCalculator<O> {
    hpo: Arc<O>,
    module_root: TermId,
}

impl<O> CohortIcCalculator<O> {
    pub fn new(hpo: Arc<O>, module_root: TermId) -> Self {
        CohortIcCalculator { hpo, module_root }
    }
}

#[derive(Debug, Default)]
struct TermCount {
    present: u32,
    excluded: u32,
}

impl<O, A, T> IcCalculator<A> for CohortIcCalculator<O>
where
    O: HierarchyWalks,
    A: Annotated<Annotation = T>,
    T: Identified + Observable,
{
    type Container = HashMap<TermId, TermIC>;

    fn compute_ic(&self, cohort: &[A]) -> Result<HashMap<TermId, TermIC>> {
        let mut module_term_ids = HashSet::new();
        module_term_ids.extend(self.hpo.iter_term_and_descendant_ids(&self.module_root));

        let mut idx2count: HashMap<_, TermCount> = HashMap::with_capacity(module_term_ids.len());

        for item in cohort {
            for annotation in item.annotations() {
                let term_id = annotation.identifier();
                if module_term_ids.contains(term_id) {
                    match annotation.observation_state() {
                        ObservationState::Present => {
                            for anc in self.hpo.iter_term_and_ancestor_ids(term_id) {
                                if module_term_ids.contains(anc) {
                                    idx2count.entry(anc).or_default().present += 1;
                                }
                            }
                        }
                        ObservationState::Excluded => {
                            for desc in self.hpo.iter_term_and_descendant_ids(term_id) {
                                /*
                                    Unlike in `ObservationState::Present` arm, we do not need
                                    to check if `desc` is contained in `module_term_ids`,
                                    since Ontology DAG guarantees this for any `term_id`
                                    contained in `module_term_ids`.
                                */
                                idx2count.entry(desc).or_default().excluded += 1;
                            }
                        }
                    }
                }
            }
        }

        if idx2count.is_empty() {
            return Ok(HashMap::new());
        }

        let pop_present_count = idx2count[&self.module_root].present as f64;

        /*
        We use max of the *entire* excluded count set,
        as opposed to just taking the max of the descendants of a `term_id` in question.
        */
        let pop_excluded_count = idx2count
            .values()
            .max_by_key(|&count| count.excluded)
            .map(|count| count.excluded)
            // We only get here if `idx2count` is not empty.
            .expect("Idx2count should not be empty") as f64;

        Ok(idx2count
            .into_iter()
            .map(|(term_id, count)| {
                (
                    Clone::clone(term_id),
                    TermIC {
                        present: f64::log2(pop_present_count / count.present as f64),
                        excluded: f64::log2(pop_excluded_count / count.excluded as f64),
                    },
                )
            })
            .collect())
    }
}
