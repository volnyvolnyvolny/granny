#![feature(hash_drain_filter)]

use std::path::Path;

mod key;
mod word;
mod password;

use crate::password::*;

/// Granny problem:
///
/// Бабушке нужно сгенерировать пароль, она слышала, что если взять четыре
/// слова из английского словаря, то можно получить хороший вариант. Но
/// проблема в том, что бабушка печатает одним пальцем и перемещать палец по
/// клавиатуре ей затруднительно, поэтому необходимо использовать такие слова,
/// которые эти перемещения минимизируют (считаются перемещения по четырём
/// сторонам, например, от "F" до "H" необходимо выполнить два перемещения, а
/// от "A" до "E" три), при том, что общая длина пароля будет от 20 до 24 символов.

fn main() {
    let words = Path::new("en_words.txt");
    let g = Goal::default();

    match Best::<Passwords>::try_from(words) {
        Ok(mut passwords) => {
            let mut sample = passwords.get_sample(40 /*sample size*/);
            let bp = sample.find_best(&g, 110 /*max_cost*/);

            println!("Found best password for a random sample: {:?}.", bp.p.words.join("|"));
            println!("Searching for the passwords that are cheaper then {} on the whole data.", bp.p.metadata.cost);

            //using it to drain_filter full data:
            let result = passwords.find_best(&g, 80/*bp.p.metadata.cost*/);
            println!("Found best password for the full dictionary!\n{:?}", result);
        },

        Err(e) =>
            panic!("{e}")
    }
}
