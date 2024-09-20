mod conftest;

use ontolius::ontology::csr::MinimalCsrOntology;
use ontolius::prelude::*;

use crate::conftest::fbn1::fbn1_ectopia_lentis_subjects;
use crate::conftest::hpo;
use infojenn::{
    ic::cohort::CohortIcCalculator,
    model::IndividualFeature,
    semsim::{ic::IcSimilarityMeasureFactory, SimilarityMeasure, SimilarityMeasureFactory},
};
use rstest::rstest;

#[rstest]
fn test_ic_smc_factory(
    hpo: MinimalCsrOntology,
    fbn1_ectopia_lentis_subjects: Vec<Vec<IndividualFeature>>,
) -> anyhow::Result<()> {
    let module_root = TermId::from(("HP", "0000118"));
    let calculator = CohortIcCalculator::new(&hpo, &module_root);
    let factory = IcSimilarityMeasureFactory::new(&hpo, calculator);

    // let measure = factory.create_measure(&fbn1_ectopia_lentis_subjects)?;

    // let left = IndividualFeature::new(TermId::from_str("HP:0001250").unwrap(), true);
    // let right = IndividualFeature::new(TermId::from_str("HP:0001250").unwrap(), true);
    // let _ = measure.compute(&left, &right);

    // bail!("to see outputs")
    Ok(())
}
