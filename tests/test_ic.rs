#[cfg(test)]
mod tests {

    use std::path::Path;

    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::CsrOntology;
    use ontolius::prelude::*;

    use infojenn::{
        data::prepare_study_subjects,
        ic::{cohort::CohortIcCalculator, IcCalculator},

    };

    #[test]
    fn test_cohort_ic_calculator() {
        let path = "/home/ielis/.hpo-toolkit/HP/hp.v2024-04-26.json";
        let hpo = load_hpo(path);

        let module_root = TermId::from(("HP", "0000118"));
        let calculator = CohortIcCalculator::new(&hpo, &module_root);
        let items = prepare_study_subjects();

        let out = calculator.compute_ic(&items);

        if let Ok(ic_container) = out {
            println!("{:?}", ic_container);
        };
    }

    fn load_hpo(
        path: impl AsRef<Path>,
    ) -> CsrOntology<usize, ontolius::base::term::simple::SimpleMinimalTerm> {
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        loader
            .load_from_path(path)
            .expect("Could not load ontology")
    }
}
