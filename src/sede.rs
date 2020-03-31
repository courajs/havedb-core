use nom::{
    combinator::map,
    bytes::complete::tag,
    branch::alt,
    multi::length_data,
    number::complete::{
        be_u32,
        be_u64,
    },
    sequence::{
        preceded,
        tuple,
    },
};

use crate::*;

pub fn length_count<I, O, N, E, F, G>(f: F, g: G) -> impl Fn(I) -> nom::IResult<I, Vec<O>, E>
where
  I: Clone,
  N: Copy + nom::ToUsize,
  F: Fn(I) -> nom::IResult<I, N, E>,
  G: Fn(I) -> nom::IResult<I, O, E>,
  E: nom::error::ParseError<I>,
{
  move |i: I| {
    let (mut i, length) = f(i)?;

    let length: usize = length.to_usize();
    let mut result = Vec::with_capacity(length);

    for _ in 0..length {
        let (i2, val) = g(i)?;
        i = i2;
        result.push(val);
    }

    Ok((i, result))
  }
}

impl FullValue {
    pub fn serialize(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.size());
        self.serialize_into(&mut v);
        v
    }

    pub fn serialize_into(&self, w: &mut Vec<u8>) {
        match self {
            FullValue::Blob(v) => {
                w.push(0);
                let len = v.len() as u64;
                w.extend_from_slice(&len.to_be_bytes()[..]);
                w.extend_from_slice(v);
            }
            FullValue::Sum(discrim, inner) => {
                w.push(1);
                w.extend_from_slice(&discrim.to_be_bytes()[..]);
                inner.serialize_into(w);
            }
            FullValue::Product(inners) => {
                w.push(2);
                let len = inners.len() as u32;
                w.extend_from_slice(&len.to_be_bytes()[..]);
                for inner in inners {
                    inner.serialize_into(w);
                }
            }
        }
    }

    pub fn deserialize(r: &[u8]) -> Result<Self, String> {
        fn parse_it(input: &[u8]) -> nom::IResult<&[u8], FullValue, nom::error::VerboseError<&[u8]>> {
            alt((
                preceded(tag([0]),
                    map(length_data(be_u64),
                    |bytes| FullValue::Blob(Vec::from(bytes)))),
                preceded(tag([1]),
                    map(tuple((be_u32, parse_it)),
                    |(discrim, inner)| FullValue::Sum(discrim, Box::new(inner)))),
                preceded(tag([2]),
                    map(length_count(be_u32, parse_it), FullValue::Product))
            ))(input)
        }

        match parse_it(r) {
            Ok((rest, it)) => {
                if rest.len() > 0 {
                    Err(format!("deserializing a FullValue left {} bytes unconsumed", rest.len()))
                } else {
                    Ok(it)
                }
            },
            Err(e) => Err(format!("{}", e))
        }
    }

    pub fn size(&self) -> usize {
        // 1 byte to distinguish enum branch, plus size of contents
        1usize + match self {
            // u64 for size of blob, plus the actual content
            FullValue::Blob(v) => 8 + v.len(),
            // u32 for discriminant
            FullValue::Sum(_, inner) => 4 + inner.size(),
            // u32 for number of fields
            FullValue::Product(inners) => {
                let sub: usize = inners.iter().map(|x|x.size()).sum();
                sub + 4
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_full_value() {
        let mut v = Vec::new();
        let val = FullValue::Product(vec![
            FullValue::Sum(4, Box::new(FullValue::Blob(vec![1, 2, 3]))),
            FullValue::Blob(vec![2,3,4,5]),
        ]);
        val.serialize_into(&mut v);

        dbg!(&v);
        assert_eq!(FullValue::deserialize(&v).unwrap(), val);
    }
}
