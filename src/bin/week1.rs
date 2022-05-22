use std::collections::HashMap;

use camino::Utf8PathBuf;
use itertools::Itertools as _;
use tap::{Pipe as _, Tap as _};

type CharacterIndex = usize;
type CipherTextIndex = usize;
type Count = u64;
type SpaceTable = HashMap<CharacterIndex, HashMap<CipherTextIndex, Count>>;

fn main() {
    let ciphertexts: Vec<Vec<u8>> = std::env::args()
        .nth(1)
        .expect("ciphertext filepath is missing")
        .pipe(Utf8PathBuf::from)
        .pipe(std::fs::read_to_string)
        .expect("Couldn't read ciphertext file")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|encoded_str| hex::decode(encoded_str).unwrap())
        .collect();

    // assume target ciphertext is at the end
    let target = ciphertexts.last().cloned().unwrap();
    let target_len = target.len();

    let mut table: SpaceTable = HashMap::new();

    for ((c1_idx, c1_str), (c2_idx, c2_str)) in ciphertexts
        .iter()
        .map(Vec::as_slice)
        .enumerate()
        .tuple_combinations::<((usize, &[u8]), (usize, &[u8]))>()
    {
        for ((idx, &c1), &c2) in c1_str.iter().enumerate().zip(c2_str).take(target_len) {
            // c1 ^ c2 = m1 ^ k ^ m2 ^ k = m1 ^ m2
            let xored = c1 ^ c2;
            // if the resulting value is alphabetic, one of the two is a space (0x20)
            // also assume both were spaces when xored value is 0 (could be wrong, but overall gives better results)
            if char::from(xored).is_alphabetic() || xored == 0 {
                // count in the space table
                let subtable = table.entry(idx).or_default();
                *subtable.entry(c1_idx).or_default() += 1;
                *subtable.entry(c2_idx).or_default() += 1;
            }
        }
    }

    let mut key = vec![0; target_len];

    for (char_idx, subtable) in table {
        println!();
        println!("========");

        println!("There is a space character (0x20) at index {char_idx} for {subtable:?}");

        let max = subtable.values().max().copied().unwrap();
        println!("The maximum number of occurrence is {max}");

        let c = subtable
            .into_iter()
            .filter(|(_, nb_occrs)| *nb_occrs == max)
            .map(|(c_idx, _)| ciphertexts[c_idx][char_idx])
            .fold(HashMap::<u8, Count>::new(), |mut acc, c| {
                *acc.entry(c).or_default() += 1;
                acc
            })
            .pipe(|acc| {
                println!("Options: {acc:?}");
                acc.into_iter()
                    .max_by(|(_, count1), (_, count2)| count1.cmp(count2))
                    .map(|(c, _)| c)
                    .unwrap()
                    .tap(|c| println!("Choose {c}"))
            });

        // we assume that `c` is the value of the space character after encryption
        // this gives use a piece of the key

        key[char_idx] = c ^ b' ';
    }

    let key_hex = hex::encode(&key);

    println!();
    println!("Found key: {key_hex}");

    let decoded: String = target
        .into_iter()
        .zip(key)
        .map(|(c, k)| char::from(c ^ k))
        .collect();

    println!();
    println!("Decoded message: {decoded}");
}
