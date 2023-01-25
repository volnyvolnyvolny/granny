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
#[derive(Debug)]
pub struct Best<T> {
    pub p : T
}

/// `HashMap` that stores passwords with the same amount of words.
pub type PasswordsLevel = HashMap<Type, Password>;

/// `HashMap` that stores all passwords.
pub type Passwords = HashMap<u8, PasswordsLevel>;

impl TryFrom<&Path> for Best<Passwords> {
    type Error = String;

    /// Loads dictionary (`words`) on the first level of the `Best<Passwords>`.
    /// We separate `4666550` words to `7177` groups (by type) and
    /// put them on the first level.
    fn try_from(words: &Path) -> Result<Self, Self::Error> {
        match File::open(words) {
            Ok(words) => {
                let mut passwords = Best{p : Passwords::new()};
                let mut level = Best{ p: PasswordsLevel::new() };

                let filter = Regex::new(r"^[a-z0-9]{2,20}$").unwrap();

                let mut count = 1;
                for word in BufReader::new(words).lines() {
                    match word {
                    	Ok(w) => {
                            if filter.is_match(w.as_str()) {
                                level.add(&Password::from(w));
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

                passwords.p.insert(1, level.p);

                Ok(passwords)
            },

            Err(e) => Err(format!("Could not open file!\n{e}"))
        }
    }
}

impl Best<PasswordsLevel> {
    /// Inserts password `p` into the *level*.
    fn insert(&mut self, p: &Password) -> &mut Self {
        let t = &p.metadata.t;
        self.p.insert(t.clone(), p.clone());
        self
    }

    /// Loads contendering password into the *level*.
    pub fn add(&mut self, c: &Password) -> &mut Self {
        let t = &c.metadata.t;

        match self.p.get(t) { //TODO: Rewrite.
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

    /// Randomly picks `n` (best) passwords and returns
    /// the new hash with them.
    pub fn get_sample(&self, n: u16) -> Self {
        if self.p.is_empty() {
            panic!("Cannot return sample from an empty HashMap.");
        }

        let mut sample = Best{ p: PasswordsLevel::new() };

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
}

impl Best<Passwords> {
    /// Drains passwords that does not match our goal.
    pub fn drain_filter(&mut self, g: &Goal) -> &mut Self {
        for (l, ps) in self.p.iter_mut() {
       	    let drained =
        	    if l == &g.words_number {
                    ps.drain_filter(
                        |t, _p| t.length < g.min_length || t.length > g.max_length
                    ).count()
        	    } else {
                    ps.drain_filter(
                        |t, _p| t.length > g.max_length
                    ).count()
                };

            if drained > 0 {
                println!("Drained {drained} passwords with {l} words (too long or too small).");
             }
        }

        self
    }

    /// Drains passwords that costs more than `max_cost`.
    pub fn drain_filter_expensive(&mut self, max_cost: u16) -> &mut Self {
        for (l, ps) in self.p.iter_mut() {
       	    let drained =
                 ps.drain_filter(
                     |_, p| p.metadata.cost >= max_cost
                 ).count();

            if drained > 0 {
                println!("Drained {drained} passwords with {l} words (too expensive).");
            }
        }

        self
    }

    /// Randomly picks `n` (best) passwords and returns
    /// the new hash with them.
    pub fn get_sample(&self, n: u16) -> Self {  
        if self.p.is_empty() {
            panic!("Cannot return sample from an empty HashMap.");
        }

        let first_level = Best{ p: self.p[&1].clone() };
        let mut sample = Best{ p: Passwords::new() };

        sample.p.insert(1, first_level.get_sample(n).p);
        sample
    }

    /// Concatenates all best passwords with `n` words with
    /// the password `p`. Propagates results to the
    /// `n + p.words.len()` level.
    ///
    /// Fills the `number1 + n2` `BestPasswords` level with concatenation
    /// of passwords from `number1` and `n2` levels.
    ///
    /// Passwords on `n` level are combination of `n` words.
    pub fn cross_levels(&mut self, number1: u8, n2: u8) -> &mut Self {
        if number1 == n2 {
            println!("Crossing {} {number1}-word passwords.", self.p[&number1].len());
        }

        let mut passwords = Best{ p: PasswordsLevel::new() };

        // Yes, it's O(n^2) :'(
        // We can do parallelization here.
        for p in self.p[&number1].values() {
            for bp in self.p[&n2].values() {
                let combined_password = p.clone() + bp.clone();

                passwords.add(&combined_password);
            }
        }

        println!("Got {} {}-word passwords.", passwords.p.len(), number1 + n2);

        self.p.insert(number1 + n2, passwords.p);
        self
    }

    /// Returns best `n` words passwords.
    fn best(&self, n: u8) -> Best<Password> {
        self.p
        .get(&n) //level n
        .and_then(
            |passwords| passwords
                        .values()
                        .min_by(|p1, p2| p1.metadata.cost.cmp(&p2.metadata.cost))
        )
        .map(|password| Best{ p: password.clone() })
        .expect("The hash is empty!")
    }

    /// Returns one best password.
    /// Calculates straightly.
    pub fn find_best(&mut self, g: &Goal, max_cost: u16) -> Best<Password> {   
        if g.words_number == 4 {
            self
            .drain_filter_expensive(max_cost)
            .cross_levels(1, /*and*/ 1)
            .drain_filter(g)
            .drain_filter_expensive(max_cost)
            .cross_levels(2, /*and*/ 2)
            .drain_filter(g)
            .drain_filter_expensive(max_cost)
            .best(g.words_number)
         } else {
         	panic!("Only 4-word passwords are supported at the moment")
         }
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
