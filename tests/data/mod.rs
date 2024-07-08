pub mod fbn1 {
    use std::str::FromStr;

    use ontolius::prelude::*;

    use infojenn::feature::IndividualFeature;

    pub fn prepare_fbn1_ectopia_lentis_subjects() -> [Box<[IndividualFeature]>; 5] {
        [
            // FBN1 -> BM
            make_study_subject(&[
                ("HP:0001083", true),
                ("HP:0001065", true),
                ("HP:0012773", true),
                ("HP:0000501", false),
                ("HP:0000545", false),
                ("HP:0000486", false),
                ("HP:0002650", false),
                ("HP:0001382", false),
                ("HP:0000767", false),
                ("HP:0001166", false),
                ("HP:0000541", false),
                ("HP:0000768", false),
                ("HP:0000218", false),
                ("HP:0002616", false),
                ("HP:0001634", false),
            ]),
            // FBN1 -> JL
            make_study_subject(&[
                ("HP:0001083", true),
                ("HP:0000545", true),
                ("HP:0001382", true),
                ("HP:0000768", true),
                ("HP:0000218", true),
                ("HP:0001065", true),
                ("HP:0000501", false),
                ("HP:0000486", false),
                ("HP:0002650", false),
                ("HP:0000767", false),
                ("HP:0001166", false),
                ("HP:0000541", false),
                ("HP:0002616", false),
                ("HP:0001634", false),
                ("HP:0012773", false),
            ]),
            // FBN1 -> OP
            make_study_subject(&[
                ("HP:0001083", true),
                ("HP:0000545", true),
                ("HP:0001166", true),
                ("HP:0000218", true),
                ("HP:0001634", true),
                ("HP:0012773", true),
                ("HP:0000501", false),
                ("HP:0000486", false),
                ("HP:0002650", false),
                ("HP:0001382", false),
                ("HP:0000767", false),
                ("HP:0000541", false),
                ("HP:0000768", false),
                ("HP:0001065", false),
                ("HP:0002616", false),
            ]),
            // FBN1 -> RWT
            make_study_subject(&[
                ("HP:0001083", true),
                ("HP:0000545", true),
                ("HP:0000486", true),
                ("HP:0001382", true),
                ("HP:0001065", true),
                ("HP:0000501", false),
                ("HP:0002650", false),
                ("HP:0000767", false),
                ("HP:0001166", false),
                ("HP:0000541", false),
                ("HP:0000768", false),
                ("HP:0000218", false),
                ("HP:0002616", false),
                ("HP:0001634", false),
                ("HP:0012773", false),
            ]),
            // FBN1 -> VW
            make_study_subject(&[
                ("HP:0001083", true),
                ("HP:0000501", true),
                ("HP:0002650", true),
                ("HP:0000218", true),
                ("HP:0001065", true),
                ("HP:0000545", false),
                ("HP:0000486", false),
                ("HP:0001382", false),
                ("HP:0000767", false),
                ("HP:0001166", false),
                ("HP:0000541", false),
                ("HP:0000768", false),
                ("HP:0002616", false),
                ("HP:0001634", false),
                ("HP:0012773", false),
            ]),
        ]
    }

    fn make_study_subject(phenotypes: &[(&str, bool)]) -> Box<[IndividualFeature]> {
        phenotypes
            .iter()
            .map(|&(curie, is_present)| {
                IndividualFeature::new(TermId::from_str(curie).unwrap(), is_present)
            })
            .collect()
    }
}
