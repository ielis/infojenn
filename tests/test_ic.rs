mod conftest;

use infojenn::model::IndividualFeature;
use ontolius::ontology::csr::MinimalCsrOntology;
use ontolius::prelude::*;
use rstest::rstest;

use crate::conftest::{fbn1::fbn1_ectopia_lentis_subjects, hpo};
use infojenn::ic::{cohort::CohortIcCalculator, IcCalculator};

#[rstest]
fn test_cohort_ic_calculator(
    hpo: MinimalCsrOntology,
    fbn1_ectopia_lentis_subjects: Vec<Vec<IndividualFeature>>,
) -> anyhow::Result<()> {
    let module_root = TermId::from(("HP", "0000118"));
    let calculator = CohortIcCalculator::new(&hpo, &module_root);

    let out = calculator.compute_ic(&fbn1_ectopia_lentis_subjects);

    if let Ok(ic_container) = out {
        println!("{:?}", ic_container);
    };

    Ok(())
}
