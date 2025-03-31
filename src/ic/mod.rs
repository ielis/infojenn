pub mod cohort;

#[derive(Debug, Clone, PartialEq)]
pub struct TermIC {
    pub present: f64,
    pub excluded: f64,
}

/// `C` - A cohort that should consist of annotated items (e.g. phenotyped individuals).
pub trait IcCalculator<C> {
    type Container;

    fn compute_ic(&self, cohort: C) -> anyhow::Result<Self::Container>;
}
