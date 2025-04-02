use ontolius::{Identified, TermId};
use phenotypes::Observable;

// A feature of a single subject.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndividualFeature {
    identifier: TermId,
    is_present: bool,
}

impl IndividualFeature {
    pub fn new(identifier: TermId, is_present: bool) -> Self {
        IndividualFeature {
            identifier,
            is_present,
        }
    }
}

impl Identified for IndividualFeature {
    fn identifier(&self) -> &TermId {
        &self.identifier
    }
}

impl Observable for IndividualFeature {
    fn is_present(&self) -> bool {
        self.is_present
    }
}
