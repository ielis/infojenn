use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use ontolius::prelude::*;
use thiserror::Error;

use super::item::AnnotatedItem;

#[derive(Debug, Error)]
pub enum IcCalculationError {
    #[error("No annotated items in the input")]
    EmptyInput,
    #[error("{0}")]
    OntologyError(Cow<'static, str>),
}

pub trait IcContainer {
    fn ic_for_term_id(&self, id: &TermId) -> Option<&f64>;

    fn len(&self) -> usize;
}

impl<'a> IcContainer for HashMap<&'a TermId, f64> {
    fn ic_for_term_id(&self, id: &TermId) -> Option<&f64> {
        self.get(id)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

pub trait IcCalculator {
    type Container: IcContainer;

    fn compute_ic<I, A>(&self, items: I) -> Result<Self::Container, IcCalculationError>
    where
        A: AnnotatedItem,
        I: IntoIterator<Item = A>;
}

pub struct HpoIcCalculator<'o, O> {
    hpo: &'o O,
    use_pseudocount: bool,
    module_root: &'o TermId,
}

impl<'o, O> HpoIcCalculator<'o, O> {
    pub fn new(hpo: &'o O, use_pseudocount: bool, module_root: &'o TermId) -> Self {
        HpoIcCalculator {
            hpo,
            use_pseudocount,
            module_root,
        }
    }
}

impl<'o, OI, O> IcCalculator for HpoIcCalculator<'o, O>
where
    OI: TermIdx + HierarchyIdx + Hash,
    O: Ontology<Idx = OI>,
{
    type Container = HashMap<&'o TermId, f64>;

    fn compute_ic<I, A>(&self, items: I) -> Result<HashMap<&'o TermId, f64>, IcCalculationError>
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

        let mut module_term_ids: HashSet<_> =
            self.hpo.hierarchy().descendants_of(module_idx).collect();
        module_term_ids.insert(&module_idx);

        let mut idx2count: HashMap<OI, u32> = HashMap::with_capacity(self.hpo.len());

        for item in items.into_iter() {
            for annotation in item.annotations() {
                if let Some(idx) = self.hpo.id_to_idx(annotation.identifier()) {
                    if module_term_ids.contains(&idx) {
                        *idx2count.entry(idx).or_insert(0) += 1;
                        for anc in self.hpo.hierarchy().ancestors_of(idx) {
                            if module_term_ids.contains(anc) {
                                *idx2count.entry(*anc).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
        if idx2count.is_empty() {
            return Err(IcCalculationError::EmptyInput);
        }

        if self.use_pseudocount {
            for item in module_term_ids {
                idx2count.entry(*item).or_insert(1);
            }
        }

        let population_count = idx2count[&module_idx];

        let term_id2ic = idx2count
            .iter()
            .map(|(idx, count)| {
                (
                    self.hpo
                        .idx_to_term_id(*idx)
                        .expect("Index was obtained from ontology so it should be there"),
                    f64::log2(population_count as f64 / *count as f64),
                )
            })
            .collect();

        Ok(term_id2ic)
    }
}
