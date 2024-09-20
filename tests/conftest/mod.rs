pub mod fbn1;

use ontolius::{io::OntologyLoaderBuilder, ontology::csr::MinimalCsrOntology};
use rstest::fixture;

#[fixture]
pub fn hpo() -> MinimalCsrOntology {
    let path = "resources/hp.v2024-08-13.json.gz";
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();

    loader.load_from_path(path).expect("Should be loadable")
}
