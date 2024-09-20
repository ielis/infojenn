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
    let phenotypic_abnormality = TermId::from(("HP", "0000118"));
    let calculator = CohortIcCalculator::new(&hpo, &phenotypic_abnormality);

    let ic_container = calculator.compute_ic(&fbn1_ectopia_lentis_subjects)?;

    assert_eq!(ic_container.len(), 178);

    // No NaNs allowed!
    assert!(!ic_container.values().any(|f|f.present.is_nan()||f.excluded.is_nan()));

    let pa_ic  = ic_container.get(&phenotypic_abnormality);
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
