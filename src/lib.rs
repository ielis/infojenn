#![doc = include_str!("../README.md")]

use std::cmp::Ordering;

use ontolius::TermId;

#[cfg(test)]
mod model;
#[cfg(test)]
mod subjects;

pub mod ic;

/// A pair of [`TermId`]s.
///
/// The term ids are maintained in a stable order
/// where *"left"* is always less than or equal to *"right"*.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermPair {
    left: TermId,
    right: TermId,
}

impl TermPair {
    /// Get the *"left"* term ID.
    pub fn left(&self) -> &TermId {
        &self.left
    }

    /// Get the *"right"* term ID.
    pub fn right(&self) -> &TermId {
        &self.right
    }
}

/// Create a pair from a term ID tuple.
///
/// ## Example
///
/// Create a term pair:
///
/// ```
/// use ontolius::TermId;
/// use infojenn::TermPair;
///
/// let left = TermId::from(("HP", "0000118"));
/// let right = TermId::from(("HP", "0001250"));
///
/// let tp = TermPair::from((left, right));
/// ```
///
/// The order of the arguments does not matter:
///
/// ```
/// # use ontolius::TermId;
/// # use infojenn::TermPair;
/// let left = TermId::from(("HP", "0000118"));
/// let right = TermId::from(("HP", "0001250"));
///
/// let lr = TermPair::from((left.clone(), right.clone()));
/// let rl = TermPair::from((right, left));
///
/// assert_eq!(lr, rl);
/// ```
impl From<(TermId, TermId)> for TermPair {
    fn from(value: (TermId, TermId)) -> Self {
        match value.0.cmp(&value.1) {
            Ordering::Less | Ordering::Equal => Self {
                left: value.0,
                right: value.1,
            },
            Ordering::Greater => Self {
                left: value.1,
                right: value.0,
            },
        }
    }
}

/// Create a term pair by cloning [`TermId`] references.
impl<'a> From<(&'a TermId, &'a TermId)> for TermPair {
    fn from(value: (&'a TermId, &'a TermId)) -> Self {
        Self::from((value.0.clone(), value.1.clone()))
    }
}

/// Create a pair representing a single [`TermId`].
impl From<TermId> for TermPair {
    fn from(value: TermId) -> Self {
        Self {
            left: Clone::clone(&value),
            right: value,
        }
    }
}
