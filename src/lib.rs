
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FullValue {
    Blob(Vec<u8>),
    Sum(u32, Box<FullValue>),
    Product(Vec<FullValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueFragment {
    Blob(Vec<u8>),
    Sum(u32, Box<ValueFragment>),
    Product(Vec<ValueFragment>),
    Reference(Hash),
}

impl FullValue {
    pub fn clone_fragment(&self) -> ValueFragment {
        match self {
            FullValue::Blob(v) => ValueFragment::Blob(v.clone()),
            FullValue::Sum(discrim, inner) => ValueFragment::Sum(*discrim, Box::new(inner.clone_fragment())),
            FullValue::Product(inners) => ValueFragment::Product(inners.iter().map(FullValue::clone_fragment).collect()),
        }
    }

    pub fn to_fragment(self) -> ValueFragment {
        match self {
            FullValue::Blob(v) => ValueFragment::Blob(v),
            FullValue::Sum(discrim, inner) => ValueFragment::Sum(discrim, Box::new(inner.to_fragment())),
            FullValue::Product(inners) => ValueFragment::Product(inners.into_iter().map(FullValue::to_fragment).collect()),
        }
    }

}

mod hashing;
pub use hashing::*;
mod sede;
pub use sede::*;
