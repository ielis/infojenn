mod data;

use std::{path::Path, str::FromStr};

use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::CsrOntology;
use ontolius::prelude::*;

use infojenn::{
    feature::IndividualFeature,
    ic::cohort::CohortIcCalculator,
    semsim::{ic::IcSimilarityMeasureFactory, SimilarityMeasure, SimilarityMeasureFactory},
};

use crate::data::fbn1::prepare_fbn1_ectopia_lentis_subjects;

#[test]
fn test_ic_smc_factory() -> anyhow::Result<()> {
    let path = "/home/ielis/.hpo-toolkit/HP/hp.v2024-04-26.json";
    let hpo = load_hpo(path);

    let module_root = TermId::from(("HP", "0000118"));
    let calculator = CohortIcCalculator::new(&hpo, &module_root);
    let factory = IcSimilarityMeasureFactory::new(&hpo, calculator);

    let items = prepare_fbn1_ectopia_lentis_subjects();
    let measure = factory.create_measure(&items)?;

    let left = IndividualFeature::new(TermId::from_str("HP:0001250").unwrap(), true);
    let right = IndividualFeature::new(TermId::from_str("HP:0001250").unwrap(), true);
    let _ = measure.compute(&left, &right);

    // bail!("to see outputs")
    Ok(())
}

fn load_hpo(
    path: impl AsRef<Path>,
) -> CsrOntology<usize, ontolius::base::term::simple::SimpleMinimalTerm> {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();

    loader
        .load_from_path(path)
        .expect("Could not load ontology")
}
