use std::ops::Deref;
use std::convert::AsRef;
use sha3::{Digest, Sha3_256};
use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn sure_from_slice(value: &[u8]) -> Hash {
        let mut val = [0; 32];
        val.copy_from_slice(value);
        Hash(val)
    }
    pub fn of(bytes: &[u8]) -> Hash {
        let mut hasher = Sha3_256::new();
        hasher.input(bytes);

        let mut val = [0; 32];
        val.copy_from_slice(hasher.result().as_ref());
        Hash(val)
    }
}
use std::fmt::{self,Debug,Display};
impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sha-256:{}", hex::encode(self.0))
    }
}
impl Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", hex::encode(&self.0[..4]))
    }
}

impl Deref for Hash {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0[..]
    }
}
impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

pub trait ContentHashed {
    fn hash(&self) -> Hash;
}

fn finish(hasher: Sha3_256) -> Hash {
    let mut val = [0; 32];
    val.copy_from_slice(hasher.result().as_ref());
    Hash(val)
}

impl ContentHashed for FullValue {
    fn hash(&self) -> Hash {
        let mut hasher = Sha3_256::new();
        match self {
            FullValue::Blob(bytes) => hasher.input(bytes),
            FullValue::Sum(discriminant, inner) => {
                hasher.input(&discriminant.to_be_bytes()[..]);
                let h = inner.hash();
                hasher.input(&h.0);
            },
            FullValue::Product(inners) => {
                for inner in inners {
                    let h = inner.hash();
                    hasher.input(&h.0);
                }
            },
        }
        finish(hasher)
    }
}

impl ContentHashed for ValueFragment {
    fn hash(&self) -> Hash {
        match self {
            ValueFragment::Blob(bytes) => {
                let mut hasher = Sha3_256::new();
                hasher.input(bytes);
                finish(hasher)
            }
            ValueFragment::Sum(discriminant, inner) => {
                let mut hasher = Sha3_256::new();
                hasher.input(&discriminant.to_be_bytes()[..]);
                let h = inner.hash();
                hasher.input(&h.0);
                finish(hasher)
            },
            ValueFragment::Product(inners) => {
                let mut hasher = Sha3_256::new();
                for inner in inners {
                    let h = inner.hash();
                    hasher.input(&h.0);
                }
                finish(hasher)
            },
            ValueFragment::Reference(h) => {
                *h
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_fragment_equivalence() {
        let blob = FullValue::Blob(vec![1,2,3]);
        let sum = FullValue::Sum(12, Box::new(blob.clone()));
        let prod = FullValue::Product(vec![blob.clone(), sum.clone()]);

        let frag = ValueFragment::Product(vec![
            ValueFragment::Reference(blob.hash()),
            ValueFragment::Reference(sum.hash()),
            ValueFragment::Reference(prod.hash()),
        ]);

        let full = FullValue::Product(vec![
            blob,
            sum,
            prod,
        ]);

        assert_eq!(frag.hash(), full.hash());
    }
}
