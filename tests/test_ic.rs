mod subjects;

use std::{fs::File, io::BufReader};

use flate2::bufread::GzDecoder;
use ontolius::{
    common::hpo::PHENOTYPIC_ABNORMALITY, io::OntologyLoaderBuilder,
    ontology::csr::MinimalCsrOntology, TermId,
};

use infojenn::ic::{cohort::CohortIcCalculator, IcCalculator};
use subjects::fbn1_ectopia_lentis_subjects;

fn load_hpo() -> MinimalCsrOntology {
    let path = "resources/hp.v2024-08-13.json.gz";

    OntologyLoaderBuilder::new()
        .obographs_parser()
        .build()
        .load_from_read(GzDecoder::new(BufReader::new(File::open(path).unwrap())))
        .expect("Should be loadable")
}

#[test]
fn test_cohort_ic_calculator() -> anyhow::Result<()> {
    let hpo = load_hpo();
    let fbn1 = fbn1_ectopia_lentis_subjects();

    let pa = PHENOTYPIC_ABNORMALITY;
    let calculator = CohortIcCalculator::new(&hpo, &pa);

    let ic_container = calculator.compute_ic(&fbn1)?;

    assert_eq!(ic_container.len(), 178);

    // No NaNs allowed!
    assert!(!ic_container
        .values()
        .any(|f| f.present.is_nan() || f.excluded.is_nan()));

    let pa_ic = ic_container.get(&PHENOTYPIC_ABNORMALITY);
    assert!(pa_ic.is_some());
    if let Some(pa_ic) = pa_ic {
        assert_eq!(pa_ic.present, 0.);
        assert_eq!(pa_ic.excluded, f64::INFINITY);
    }

    let myopia = TermId::from(("HP", "0000545"));
    let myopia_ic = ic_container.get(&myopia);
    assert!(myopia_ic.is_some());
    if let Some(myopia_ic) = myopia_ic {
        assert_eq!(myopia_ic.present, 3.0588936890535687);
        assert_eq!(myopia_ic.excluded, 1.3219280948873624);
    }

    let ectopia_lentis = TermId::from(("HP", "0001083"));
    let el_ic = ic_container.get(&ectopia_lentis);
    assert!(el_ic.is_some());
    if let Some(el_ic) = el_ic {
        assert_eq!(el_ic.present, 2.3219280948873622);
        assert_eq!(el_ic.excluded, f64::INFINITY);
    }

    Ok(())
}
