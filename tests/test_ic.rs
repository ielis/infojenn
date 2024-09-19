mod data;

use ontolius::prelude::*;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::MinimalCsrOntology};

use crate::data::fbn1::prepare_fbn1_ectopia_lentis_subjects;
use infojenn::ic::{cohort::CohortIcCalculator, IcCalculator};

#[test]
fn test_cohort_ic_calculator() -> anyhow::Result<()> {
    let path = "resources/hp.v2024-08-13.json.gz";
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();

    let hpo: MinimalCsrOntology = loader.load_from_path(path)?;

    let module_root = TermId::from(("HP", "0000118"));
    let calculator = CohortIcCalculator::new(&hpo, &module_root);
    let items = prepare_fbn1_ectopia_lentis_subjects();

    let out = calculator.compute_ic(&items);

    if let Ok(ic_container) = out {
        println!("{:?}", ic_container);
    };

    Ok(())
}
