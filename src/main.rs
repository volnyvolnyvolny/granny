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
        Ok(passwords) => {
            let sample_size = 100;
            let max_cost = 200;

            println!("\n> Taking a random sample: {sample_size} of the 1-word best passwords.");

            let sample = passwords.get_sample(sample_size);
            let mut best = sample.find_best(&g, max_cost);

            let max_cost = best.p.metadata.cost;
            println!("\n= The best password for a sample: {:?} (len: {}, cost: {}).", best.p.words.join("|"), best.p.metadata.t.length, best.p.metadata.cost);

            println!("> Searching for the passwords that are cheaper then {max_cost} on the whole data.\n");

            // using it to filter full data:
            best = passwords.find_best(&g, max_cost);
            println!("= The best password for the full dictionary!\n{:?} (len: {}, cost: {}).", best.p.words.join("|"), best.p.metadata.t.length, best.p.metadata.cost);
       },

        Err(e) =>
            panic!("{e}")
    }
}
