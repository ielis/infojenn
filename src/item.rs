use ontolius::base::Identified;

use crate::feature::{FrequencyAware, Observable};

pub trait AnnotatedItem {
    type Annotation: Identified + FrequencyAware;

    fn annotations(&self) -> &[Self::Annotation];

    fn present_annotations(&self) -> impl Iterator<Item = &Self::Annotation> {
        self.annotations().iter().filter(|&a| a.is_present())
    }

    fn excluded_annotations(&self) -> impl Iterator<Item = &Self::Annotation> {
        self.annotations().iter().filter(|&a| a.is_excluded())
    }
}

impl<T> AnnotatedItem for Box<[T]>
where
    T: Identified + FrequencyAware,
{
    type Annotation = T;

    fn annotations(&self) -> &[Self::Annotation] {
        self
    }
}
