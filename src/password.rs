use rand::seq::SliceRandom;

use crate::word::*;
use crate::word::End;
use crate::word::End::*;

use regex::Regex;

use std::collections::HashMap;

use std::convert::TryFrom;
use std::convert::From;

use std::ops::Add;
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader};

use std::cmp::Ordering;

/// Represents *search goal*.
/// Defaults to `4` words, `20-24` symbols password.
#[derive(Clone, Copy, Debug)]
pub struct Goal {
    pub words_number: u8,
    pub min_length: u8,
    pub max_length: u8
}

impl Default for Goal {
    fn default() -> Goal {
        Goal {
            words_number: 4,
            min_length: 20,
            max_length: 24
        }
    }
}

/// Represents *password*.
///
/// Passwords `metadata` contains `t: Type{ first_key, last_key, length }` and `cost`.
///
/// For two passwords having the same type we should choose the cheapest one.
///
/// # Example
///
/// ````
/// Password { words: vec!["granny".to_string()], metadata: Metadata{ t: Type{ first_key: 'G', last_key: 'Y', length: 6 }, cost: 14 } }
/// ````
#[derive(Clone,Debug)]
pub struct Password {
    pub words: Vec<String>,
    pub metadata: Metadata
}

/// Simple arithmetic on passwords.
///
/// # Examples
///
/// ````
/// Password { words: vec!("granny".to_string()), metadata: Metadata{ t: Type{ first_key: 'G', last_key: 'Y', length: 6 }, cost: 14 } }
/// //+
/// Password { words: vec!("panties".to_string()), metadata: Metadata{ t: Type{ first_key: 'P', last_key: 'S', length: 7 }, cost: 29 } }
/// //==
/// Password { words: vec!("granny".to_string(), "panties".to_string()), metadata: Metadata{ t: Type{ first_key: 'G', last_key: 'S', length: 13 }, cost: 47 } }
/// ````
impl Add for Password {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let sum = [
                self.words.as_slice(),
                other.words.as_slice()
            ].concat();

        let joined = sum.join("");

        Password {
            words: sum,
            metadata: Metadata::from(joined)
        }
    }
}

impl Eq for Password {}

/// Two passwords are equal if their string representation are equal.
impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.words.concat() == other.words.concat()
    }
}

impl PartialOrd for Password {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {   
        if self.words.len() == other.words.len() {
            let m1 = &self.metadata;
            let m2 = &other.metadata;

        	Some(m1.cost.cmp(&m2.cost))
        } else {
            None
        }
    }
}

impl From<String> for Password {
    fn from(word: String) -> Self {
        Password {
	        words: vec!(word.clone()),
            metadata: Metadata::from(word)
        }
    }
}

/// Wrapper to compose `Best<Password>` and `Best<Passwords>`.
#[derive(Clone, Debug)]
pub struct Best<T> {
    pub p : T
}

impl Best<Password> {
    /// Binds password from the left or right end.
	pub fn bind(&mut self, end: &End) -> &mut Self {
	    self.p.metadata.bind(end);
	    self
	}

    /// Returns if password is binded from the left or right end.
	pub fn is_binded(&self, end: &End) -> bool {
	    self.p.metadata.is_binded(end)
	}
}

/// Arithmetics for `Best<Password>`.
impl Add for Best<Password> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let sum = self.p.clone() + other.p.clone();
    
        let mut sum = Best{ p: sum };

        if self.is_binded(&End::Left) {
            sum.bind(&Left);
        }

        if other.is_binded(&End::Right) {
            sum.bind(&Right);
        }

        sum
    }
}

/// Comparing `Best<Password>` and `Best<Password>`
impl PartialEq for Best<Password> {
    fn eq(&self, other: &Self) -> bool {
        self == &other.p
     && self.p.metadata == other.p.metadata
    }
}

/// Comparing `Best<Password>` and `Password`
impl PartialEq<Password> for Best<Password> {
    fn eq(&self, other: &Password) -> bool {
        &self.p == other
    }
}

/// Comparing `Best<Password>` and `Password`
impl PartialOrd<Password> for Best<Password> {
    fn partial_cmp(&self, other: &Password) -> Option<Ordering> {
        self.p.partial_cmp(other)
    }
}

/// `HashMap` that stores passwords with the same amount of words.
pub type Passwords = HashMap<Type, Best<Password>>;

impl TryFrom<&Path> for Best<Passwords> {
    type Error = String;

    /// Loads dictionary (`words`) to form `Best<Passwords>` hash.
    /// For example, we use `466 550` english words to form `7177` best 1-word
    /// passwords (by type).
    fn try_from(words: &Path) -> Result<Self, Self::Error> {
        match File::open(words) {
            Ok(words) => {
                let mut passwords = Best{p : Passwords::new()};

                let filter = Regex::new(r"^[a-z0-9]{3,20}$").unwrap();

                let mut count = 1;
                for word in BufReader::new(words).lines() {
                    match word {
                    	Ok(w) => {
                    	    let ws = w.as_str();

                            if filter.is_match(w.as_str())
                            && ws != "oooo" && ws != "mmmm" && ws != "xxx"
                            && ws != "eer"
                            && ws != "dedd"
                            {
                                passwords.push(&Password::from(w));
                            }
                    	},
                    	Err(e) => {
                    	    println!("Could not load word!\n{e}");
                            break;
                        }
                    }
                    count += 1;
                }

                println!("Loaded {count} words.");

                Ok(passwords)
            },

            Err(e) => Err(format!("Could not open file!\n{e}"))
        }
    }
}

/// Arithmetics for `Best<Password>`.
impl Add<&Best<Passwords>> for Best<Passwords> {
    type Output = Self;

    /// Concatenates best passwords from one hash with
    /// best password of another hash.
    fn add(self, other: &Self) -> Self {
        println!("[Concat]\tConcatenating {} and {} passwords.", self.p.len(), other.p.len());
    
        let mut passwords = Best{ p: Passwords::new() };

        for p in self.p.values() {
            for bp in other.p.values() {
                let combined = p.clone() + bp.clone();

                passwords.push(&combined.p);
            }
        }

        println!("[Concat]\tGot {} passwords.", passwords.p.len());
        passwords
    }
}

impl Best<Passwords> {
    /// Inserts `winner` password into the hash.
    fn insert_winner(&mut self, w: &Password) -> &mut Self {
        let t = &w.metadata.t;
        self.p.insert(t.clone(), Best{ p: w.clone() });
        self
    }

    /// Loads contendering password into the hash.
    /// If it is cheaper than existing one
    /// -- make a replace.
    pub fn push(&mut self, c: &Password) -> &mut Self {
        let t = &c.metadata.t;

        match self.p.get(t) {
            None =>
                self.insert_winner(c),

            Some(bp) =>
                if bp <= c {
                    self
                } else {
                    self.insert_winner(c)
                }
        }
    }

    /// Binds best passwords from the left or right end.
    pub fn bind(&self, end: &End) -> Self {
        let mut passwords = Best{ p: Passwords::new() };

        for p in self.p.values() {
            let mut b = p.clone();

            b.bind(end);

            passwords.push(&b.p);
        }

        println!("[Bind {:?}]\tGot {} passwords.", end, passwords.p.len());
        passwords
    }

    /// Randomly picks `n` (best) passwords and returns
    /// the new hash with them.
    pub fn get_sample(&self, n: u16) -> Self {
        if self.p.is_empty() {
            panic!("Cannot return sample from an empty HashMap.");
        }

        let mut sample = Best{ p: Passwords::new() };

        let all_keys : Vec<_> =
            self.p
            .clone()
            .into_keys()
            .collect();

        let keys =
            all_keys
            .choose_multiple(&mut rand::thread_rng(), n.into());

        for t in keys {
        	sample.insert_winner(&self.p[t].p);
        }

        sample
    }

    /// Drains passwords that does not match our goal.
    pub fn filter_by_length(&mut self, g: &Goal) -> &mut Self {
   	    let drained =
            self.p.drain_filter(
                |t, best| {
                    let is_too_long  = t.length > g.max_length;
                    let is_too_small = t.length < g.min_length;

                    if best.p.words.len() as u8 == g.words_number {
                        is_too_long || is_too_small
                    } else {
                        is_too_long
                    }
                }
            ).count();

        if drained > 0 {
            println!("[Filter]\tDropped {drained} passwords (too long or too small).");
        }

        self
    }
 
    /// Drains passwords that costs more than `max`.
    pub fn filter_by_cost(&mut self, max: u16) -> &mut Self {
   	    let drained =
            self.p.drain_filter(
                |_, best| best.p.metadata.cost > max
            ).count();

        if drained > 0 {
            println!("[Filter]\tDropped {drained} passwords (expensive).");
        }

        self
    }

    /// Drains passwords that costs more than `max` or does not match our goal.
    pub fn filter(&mut self, g: &Goal, max_cost: u16) -> &mut Self {
        self.filter_by_cost(max_cost)
            .filter_by_length(g)
    }

    /// Returns the best of the best password.
    pub fn best(&self) -> Best<Password> {
        self.p
            .values()
            .min_by(|b1, b2| b1.p.metadata.cost.cmp(&b2.p.metadata.cost))
            .expect("The hash is empty!")
            .clone()
    }

    /// Returns the best password.
    pub fn find_best(&self, g: &Goal, max_cost: u16) -> Best<Password> {
        println!("\n* Making 1-word passwords (=left binded):\n");

        let mut left  = self.bind(&End::Left);  //passwords that starts with any ('*') key
        left.filter(g, max_cost);

        let mut acc = left;

        if g.words_number >= 2 {
            let mut center = self.clone();
            center.filter(g, max_cost);

            for i in 2..g.words_number {
                println!("\n* Making {i}-word passwords:\n");
                acc = acc + &center;
 
          	    acc.filter(g, max_cost);
            }

            println!("\n* Making {}-word passwords (+right binded):\n", g.words_number);
            let mut right = self.bind(&End::Right); //passwords that ends with any key
            right.filter(g, max_cost);

            acc = acc + &right;
            acc.filter(g, max_cost);
        }

        acc.filter(g, max_cost).best()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass() {
        let g = Password::from("granny".to_string());
        let p = Password::from("panties".to_string());
    
        assert_eq!(
          Password::from("granny".to_string()),
          Password { metadata: Metadata{ t: Type{ first_key: 'G', last_key: 'Y', length: 6}, cost: 14 }, words: vec!("granny".to_string()) }
        );

        assert_eq!(
          Password::from("panties".to_string()),
          Password { metadata: Metadata{ t: Type{ first_key: 'P', last_key: 'S', length: 7 }, cost: 29 }, words: vec!("panties".to_string()) }
        );

        assert_eq!(
          g + p,
          Password { metadata: Metadata{ t: Type{ first_key: 'G', last_key: 'S', length: 13 }, cost: 47 }, words: vec!("granny".to_string(), "panties".to_string()) }
        );

        let zack = Password { metadata: Metadata{ t: Type{ first_key: 'Z', last_key: 'K', length: 4}, cost: 10 }, words: vec!("zack".to_string()) };
        let miri = Password { metadata: Metadata{ t: Type{ first_key: 'M', last_key: 'I', length: 4}, cost: 11 }, words: vec!("miri".to_string()) };

        assert_eq!(
          zack + miri,
          Password { metadata: Metadata{ t: Type{ first_key: 'M', last_key: 'I', length: 8}, cost: 22 }, words: vec!("zack".to_string(), "miri".to_string()) }
        );
    }
}
