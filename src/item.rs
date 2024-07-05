use crate::feature::{Feature, Observable};

pub trait AnnotatedItem {
    type Annotation: Feature;

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
    T: Feature,
{
    type Annotation = T;

    fn annotations(&self) -> &[Self::Annotation] {
        self
    }
}
