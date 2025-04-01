use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use ontolius::{ontology::HierarchyWalks, Identified, TermId};
use phenotypes::Observable;

#[derive(Debug, Clone, PartialEq)]
pub struct TermIC {
    pub present: f64,
    pub excluded: f64,
}

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

impl<O> CohortIcCalculator<O>
where
    O: HierarchyWalks,
{
    pub fn compute_ic<C, M, A>(&self, cohort: C) -> HashMap<TermId, TermIC>
    where
        C: AsRef<[M]>,
        M: AsRef<[A]>,
        A: Identified + Observable,
    {
        let mut module_term_ids = HashSet::new();
        module_term_ids.extend(self.hpo.iter_term_and_descendant_ids(&self.module_root));

        let mut idx2count: HashMap<_, TermCount> = HashMap::with_capacity(module_term_ids.len());

        for member in cohort.as_ref() {
            for annotation in member.as_ref() {
                let term_id = annotation.identifier();
                if module_term_ids.contains(term_id) {
                    if annotation.is_present() {
                        for anc in self.hpo.iter_term_and_ancestor_ids(term_id) {
                            if module_term_ids.contains(anc) {
                                idx2count.entry(anc).or_default().present += 1;
                            }
                        }
                    } else {
                        for desc in self.hpo.iter_term_and_descendant_ids(term_id) {
                            /*
                                Unlike in `is_present` arm, we do not need
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

        if idx2count.is_empty() {
            return HashMap::new();
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

        idx2count
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
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, sync::Arc};

    use flate2::bufread::GzDecoder;
    use ontolius::{
        common::hpo::PHENOTYPIC_ABNORMALITY, io::OntologyLoaderBuilder,
        ontology::csr::MinimalCsrOntology, TermId,
    };

    use crate::{ic::CohortIcCalculator, subjects::fbn1_ectopia_lentis_subjects};

    fn load_hpo() -> MinimalCsrOntology {
        let path = "resources/hp.v2024-08-13.json.gz";

        OntologyLoaderBuilder::new()
            .obographs_parser()
            .build()
            .load_from_read(GzDecoder::new(BufReader::new(File::open(path).unwrap())))
            .expect("Should be loadable")
    }

    #[test]
    fn test_cohort_ic_calculator() {
        let hpo = Arc::new(load_hpo());
        let fbn1 = fbn1_ectopia_lentis_subjects();

        let pa = PHENOTYPIC_ABNORMALITY;
        let calculator = CohortIcCalculator::new(hpo, pa);

        let ic_container = calculator.compute_ic(&fbn1);

        assert_eq!(ic_container.len(), 178);

        // No NaNs allowed!
        assert!(!ic_container
            .values()
            .any(|term_ic| term_ic.present.is_nan() || term_ic.excluded.is_nan()));

        let pa_ic = ic_container.get(&PHENOTYPIC_ABNORMALITY);
        assert!(pa_ic.is_some());
        if let Some(pa_ic) = pa_ic {
            assert_eq!(pa_ic.present, 0.);
            assert_eq!(pa_ic.excluded, f64::INFINITY);
        }

        let myopia: TermId = "HP:0000545".parse().unwrap();
        let myopia_ic = ic_container.get(&myopia);
        assert!(myopia_ic.is_some());
        if let Some(myopia_ic) = myopia_ic {
            assert_eq!(myopia_ic.present, 3.0588936890535687);
            assert_eq!(myopia_ic.excluded, 1.3219280948873624);
        }

        let ectopia_lentis: TermId = "HP:0001083".parse().unwrap();
        let el_ic = ic_container.get(&ectopia_lentis);
        assert!(el_ic.is_some());
        if let Some(el_ic) = el_ic {
            assert_eq!(el_ic.present, 2.3219280948873622);
            assert_eq!(el_ic.excluded, f64::INFINITY);
        }
    }
}
