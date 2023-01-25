use crate::key::*;

/// Represents `word type`. 
#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct Type {
    pub first_key: Key,
    pub last_key: Key,
    pub length: u8
}

/// Represents metadata for a word.
/// Contains `Type` and `cost`.
#[derive(Clone, Debug)]
pub struct Metadata {
	pub t: Type,
	pub cost: Distance
}

///Calculates `word` cost.
///
///# Examples
///
/// ````
/// assert_eq!(cost("granny"), 14) //g -2> r -4> a -6> n -0> n -2> y
/// ````
pub fn cost(word: String) -> Distance {
    word.chars()
    .fold(
        (0, to_key(word.chars().next().expect("Empty string was given"))),
        |(d, last_k), c| {
            let k = to_key(c);
            (d + distance(last_k, k), k)
        }
      )
    .0
}

impl From<String> for Metadata {
    fn from(word: String) -> Self {
        Metadata {
	        t: Type::from(word.clone()),
            cost: cost(word)
        }
    }
}

impl From<String> for Type {
    fn from(word: String) -> Self {
        let mut chars = word.chars();

        Type {
            first_key: to_key(chars.next().expect("Empty string was given")),
            last_key: to_key(chars.last().expect("Empty string was given")),
            length: word.len() as u8
        }
    }

}
