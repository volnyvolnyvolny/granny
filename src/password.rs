use rand::seq::SliceRandom;

use crate::word::*;

use regex::Regex;

use std::collections::HashMap;

use std::convert::TryFrom;
use std::convert::From;

use std::ops::Add;
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader};

use std::cmp::Ordering;
use std::cmp::Ordering::*;

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
#[derive(Clone, Debug)]
pub struct Password {
    pub words: Vec<String>,
    pub metadata: Metadata
}

/// Represents *search goal*.
/// Defaults to `4` words, `20-24` symbols password.
#[derive(Clone, Debug)]
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

pub enum End {
	Left,
	Right
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
    type Output = Password;

    fn add(self, other: Password) -> Password {
        let combination = [
                self.words.as_slice(),
                other.words.as_slice()
            ].concat();

        let joined = combination.join("");

        Password {
            words: combination,
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

/// Wrapper to compose `Best<Password>`, `Best<Passwords>` and `Best<PasswordsLevel>` types.
#[derive(Clone, Debug)]
pub struct Best<T> {
    pub p : T
}

/// `HashMap` that stores passwords with the same amount of words.
pub type Passwords = HashMap<Type, Password>;

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
                            if filter.is_match(w.as_str()) {
                                passwords.add(&Password::from(w));
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

impl Best<Passwords> {
    /// Inserts password `p` into the hash.
    fn insert(&mut self, p: &Password) -> &mut Self {
        let t = &p.metadata.t;
        self.p.insert(t.clone(), p.clone());
        self
    }

    /// Loads contendering password into the hash.
    /// If it's better than existing one
    /// -- make a replace.
    pub fn add(&mut self, c: &Password) -> &mut Self {
        let t = &c.metadata.t;

        match self.p.get(t) {
            None =>
                self.insert(c),

            Some(p) =>
                if p.partial_cmp(c) == Some(Greater) {
                    self
                } else {
                    self.insert(c)
                }
        }
    }

    /// Concatenates best passwords from one hash with
    /// best password of another hash.
    pub fn mult(&self, other: &Self) -> Self {
        println!("[Mult] Multiplying {} and {} passwords.", self.p.len(), other.p.len());
    
        let mut passwords = Best{ p: Passwords::new() };

        for p in self.p.values() {
            for bp in other.p.values() {               
                let combined_password = p.clone() + bp.clone();

                passwords.add(&combined_password);
            }
        }

        println!("[Mult] Got {} passwords.", passwords.p.len());
        passwords
    }

    /// Binds best passwords from the left or right side.
    pub fn bind(&self, from: &End) -> Self {
        let mut passwords = Best{ p: Passwords::new() };

        for p in self.p.values() {
            let mut p = p.clone();

            match from {
                End::Left => p.metadata.t.first_key = '*',
                End::Right => p.metadata.t.last_key = '*'
            }

            passwords.add(&p);
        }

        println!("[Bind] Got {} passwords.", passwords.p.len());
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

        for k in keys {
        	sample.insert(&self.p[k]);
        }

        sample
    }

    /// Drains passwords that does not match our goal.
    pub fn filter_by_length(&mut self, g: &Goal) -> &mut Self {
   	    let drained =
            self.p.drain_filter(
                |t, p| {
                    let is_too_long  = t.length > g.max_length;
                    let is_too_small = t.length < g.min_length;

                    if p.words.len() as u8 == g.words_number {
                        is_too_long || is_too_small
                    } else {
                        is_too_long
                    }
                }
            ).count();

        if drained > 0 {
            println!("[Filter] Dropped {drained} passwords (too long or too small).");
        }

        self
    }
 
    /// Drains passwords that costs more than `max`.
    pub fn filter_by_cost(&mut self, max: u16) -> &mut Self {
   	    let drained =
            self.p.drain_filter(
                |_, p| p.metadata.cost >= max
            ).count();

        if drained > 0 {
            println!("[Filter] Dropped {drained} passwords (too expensive).");
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
            .min_by(|p1, p2| p1.metadata.cost.cmp(&p2.metadata.cost))
            .map(|password| Best{ p: password.clone() })
            .expect("The hash is empty!")
    }

    /// Returns the best password.
    pub fn find_best(&self, g: &Goal, max_cost: u16) -> Best<Password> {
        let mut center = self.clone();
        center.filter(g, max_cost);

        let mut left  = self.bind(&End::Left);  //passwords that starts with any ('*') key
        let mut right = self.bind(&End::Right); //passwords that ends with any key

        left.filter(g, max_cost);
        right.filter(g, max_cost);

        let mut acc = left.mult(&center);
        acc.filter(g, max_cost);

        for _ in 2 .. g.words_number {
          	acc = acc.mult(&center);

          	acc.filter(g, max_cost);
        }

        acc.mult(&right).best()
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
    }
}
