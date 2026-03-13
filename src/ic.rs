use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use ontolius::{ontology::HierarchyWalks, Identified, TermId};
use phenotypes::Observable;

use crate::TermPair;

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

    /// Compute information content of the most informative common ancestor (IC<sub>MICA</sub>) for term pairs from the `cohort`.
    ///
    /// Returns a map with [`TermPair`]s as keys and IC<sub>MICA</sub> as values.
    /// The map does *NOT* contain unrelated term entries (i.e. those with IC<sub>MICA</sub> equal to `0`).
    /// The map contains only the term pairs for the term (plus their ancestors) observed in at least one `cohort` member.
    pub fn compute_ic_mica<C, M, A>(&self, cohort: C) -> HashMap<TermPair, f64>
    where
        C: AsRef<[M]>,
        M: AsRef<[A]>,
        A: Identified + Observable,
    {
        let mut cohort_ig = HashSet::new();
        for member in cohort.as_ref() {
            for pf in member.as_ref() {
                if pf.is_present() {
                    cohort_ig.extend(
                        self.hpo
                            .iter_term_and_ancestor_ids(pf.identifier())
                            .cloned(),
                    );
                }
            }
        }

        let ic = self.compute_ic(cohort);
        let mut module = Vec::new();

        let mut ic_micas = HashMap::new();
        for module_root in self.hpo.iter_child_ids(&self.module_root) {
            module.extend(
                self.hpo
                    .iter_term_and_descendant_ids(module_root)
                    .filter(|ti| cohort_ig.contains(ti)),
            );

            for (i, &left) in module.iter().enumerate() {
                for &right in &module[i..] {
                    if let Some(ic_mica) = common_ancestors(left, right, self.hpo.as_ref())
                        .into_iter()
                        .flat_map(|t| ic.get(t).map(|tic| tic.present))
                        .filter(|&f| f > 0.)
                        .reduce(f64::max)
                    {
                        let key = TermPair::from((left, right));
                        ic_micas
                            .entry(key)
                            .and_modify(|val: &mut f64| *val = val.max(ic_mica))
                            .or_insert(ic_mica);
                    }
                }
            }

            module.clear();
        }

        ic_micas
    }
}

fn common_ancestors<'a, O>(left: &'a TermId, right: &'a TermId, hpo: &'a O) -> Vec<&'a TermId>
where
    O: HierarchyWalks,
{
    let work: Vec<_> = hpo.iter_term_and_ancestor_ids(left).collect();

    hpo.iter_term_and_ancestor_ids(right)
        .filter(|x| work.contains(x))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::BufReader,
        sync::{Arc, OnceLock},
    };

    use flate2::bufread::GzDecoder;
    use ontolius::{
        common::hpo::PHENOTYPIC_ABNORMALITY, io::OntologyLoaderBuilder,
        ontology::csr::MinimalCsrOntology, TermId,
    };

    use crate::{
        ic::{CohortIcCalculator, TermPair},
        subjects::fbn1_ectopia_lentis_subjects,
    };

    static HPO: OnceLock<Arc<MinimalCsrOntology>> = OnceLock::new();

    fn load_hpo() -> Arc<MinimalCsrOntology> {
        let path = "resources/hp.v2024-08-13.json.gz";
        Arc::new(
            OntologyLoaderBuilder::new()
                .obographs_parser()
                .build()
                .load_from_read(GzDecoder::new(BufReader::new(File::open(path).unwrap())))
                .expect("Should be loadable"),
        )
    }

    #[test]
    fn test_cohort_ic_calculator() {
        let hpo = Arc::clone(HPO.get_or_init(load_hpo));
        let fbn1 = fbn1_ectopia_lentis_subjects();

        let pa = PHENOTYPIC_ABNORMALITY.clone();
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

    #[test]
    fn test_compute_ic_mica() {
        let hpo = Arc::clone(HPO.get_or_init(load_hpo));
        let fbn1 = fbn1_ectopia_lentis_subjects();
        let cic = CohortIcCalculator::new(hpo, PHENOTYPIC_ABNORMALITY.clone());

        let ic_mica = cic.compute_ic_mica(&fbn1);

        // Test some terms
        let ectopia_lentis: TermId = "HP:0001083".parse().unwrap();
        assert_eq!(
            ic_mica.get(&TermPair::from(ectopia_lentis.clone())),
            Some(&2.3219280948873622),
        );

        let myopia: TermId = "HP:0000545".parse().unwrap();
        assert_eq!(
            ic_mica.get(&TermPair::from(myopia.clone())),
            Some(&3.0588936890535687),
        );

        let abn_eye_physiology: TermId = "HP:0012373".parse().unwrap();
        assert_eq!(
            ic_mica.get(&TermPair::from(abn_eye_physiology.clone())),
            Some(&2.321928094887362),
        );

        let abn_of_the_eye: TermId = "HP:0000478".parse().unwrap();
        assert_eq!(
            ic_mica.get(&TermPair::from(abn_of_the_eye.clone())),
            Some(&1.3219280948873624),
        );

        assert_eq!(
            ic_mica.get(&TermPair::from((&myopia, &ectopia_lentis))),
            Some(&1.3219280948873624),
        );

        let striae_distensae: TermId = "HP:0001065".parse().unwrap();
        assert_eq!(
            ic_mica.get(&TermPair::from(striae_distensae.clone())),
            Some(&2.643856189774725),
        );

        assert_eq!(
            // No zero entries
            ic_mica.get(&TermPair::from(PHENOTYPIC_ABNORMALITY.clone())),
            None,
        );

        assert_eq!(
            // No common ancestor
            ic_mica.get(&TermPair::from((
                striae_distensae.clone(),
                ectopia_lentis.clone()
            ))),
            None,
        );
    }
}
