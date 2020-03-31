
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

mod hashing;
pub use hashing::*;
mod sede;
pub use sede::*;
