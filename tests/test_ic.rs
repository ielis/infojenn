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

    let ic_container = calculator.compute_ic(&fbn1_ectopia_lentis_subjects)?;

    assert_eq!(ic_container.len(), 178);
    let myopia = TermId::from(("HP", "0000545"));
    let myopia_ic = ic_container.get(&myopia);
    assert!(myopia_ic.is_some());
    
    let myopia_ic = myopia_ic.unwrap();
    assert_eq!(myopia_ic.present, 3.0588936890535687);
    assert_eq!(myopia_ic.excluded, 1.3219280948873624);

    Ok(())
}
