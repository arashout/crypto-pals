use std::collections::HashMap;

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;

mod utils;
mod frequency_analysis;

use utils::{from_hex, to_hex, hex_to_base64};
use frequency_analysis::{hamming};

#[macro_use]
extern crate lazy_static;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

lazy_static! {
    static ref ENGLISH_FREQ: HashMap<char, f64> = hashmap!(
        'E' => 0.1202,'T' => 0.0910,'A' => 0.0812,'O' => 0.0768,'I' => 0.0731,'N' => 0.0695,'S' => 0.0628,'R' => 0.0602,
        'H' => 0.0592,'D' => 0.0432,'L' => 0.0398,'U' => 0.0288,'C' => 0.0271,'M' => 0.0261,'F' => 0.0230,'Y' => 0.0211,'W' => 0.0209,
        'G' => 0.0203,'P' => 0.0182,'B' => 0.0149,'V' => 0.0111,'K' => 0.0069,'X' => 0.0017,'Q' => 0.0011,'J' => 0.0010,'Z' => 0.0007
    );
}
fn main() -> std::io::Result<()> {
    let mut file = File::open("S1C6decoded.txt")?;
    let mut buffer = Vec::new();
    // read the whole file
    file.read_to_end(&mut buffer)?;

    // let input = base64::decode(&buffer).unwrap();
    let keysizes = guess_keysizes(&buffer);

    let best_keysize = keysizes.get(0).map(|t| t.0).unwrap();

    // For each element of key, need to do frequency analysis
    let mut transposed: Vec<Vec<u8>> = Vec::with_capacity(best_keysize);
    for _ in 0..best_keysize{
        transposed.push(Vec::new());
    }

    buffer
        .windows(best_keysize)
        .for_each(|window| {
            for (j, n) in window.iter().enumerate() {
                transposed[j].push(*n)
            }
    });

    for (i, t_elements) in transposed.iter().enumerate() {
        let res_t: Vec<(f64, u8)> = find_decode_xor(t_elements).iter().take(5).map(|x|x.clone()).collect();
        println!("{} {:?}\n", i,  &res_t);
    }
    Ok(())
}

const MAX_KEYSIZE: usize = 40;
fn guess_keysizes(input: &[u8]) -> Vec<(usize, f64)> {
    let mut res = Vec::new();

    for ks in 2..MAX_KEYSIZE {
        let mut scores: Vec<f64> = Vec::new();
        assert!(input.len()/ks > 2);
        // TODO: Need to double check that this is legit
        for (a,b) in input.chunks_exact(ks).zip(input.chunks_exact(ks).skip(1)){
            scores.push( (hamming(a,b) as f64)/ks as f64  );
        }

        res.push((ks, scores.iter().fold(0.0, |acc, x| acc + x)/scores.len() as f64 ));
    }
    res.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    res
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn repeat_xor(input: &[u8], key: &[u8]) -> Vec<u8> {
    // 1. split the input into chunks of size key.len()
    // 2. xor element_in_chunk element_in_key
    // 3. convert back to Vec<u8>
    input
        .chunks(key.len())
        .map(|chunk| chunk.iter().zip(key).map(|(b, k)| b ^ k))
        .fold(Vec::new(), |mut acc, chunk| {
            chunk.for_each(|b| acc.push(b));
            acc
        })
}

pub fn find_decode_xor(b: &[u8]) -> Vec<(f64, u8)> {
    let mut res_vector = Vec::new();

    for n in 1..=127 {
        let decrypted: Vec<u8> = b.iter().map(|x| x ^ n).collect();
        let res = std::str::from_utf8(&decrypted);
        if res.is_err() {
            continue;
        }
        let english = res.unwrap();
        let mut freq = HashMap::new();
        let mut total = 0;
        for c in english.chars() {
            total += 1;
            let counter = freq.entry(c.to_ascii_uppercase()).or_insert(0.0);
            *counter += 1.0;
        }

        for (k, _v) in ENGLISH_FREQ.iter() {
            let counter = freq.entry(*k).or_insert(0.0);
            *counter = *counter / total as f64;
        }
        let sim = similarity(&ENGLISH_FREQ, &freq);
        res_vector.push((sim, n));
    }

    res_vector.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    res_vector.reverse();
    res_vector
}



// Assuming Hashmap b is a subset of hashmap a
pub fn similarity<T>(a: &HashMap<T, f64>, b: &HashMap<T, f64>) -> f64
where
    T: std::cmp::Eq,
    T: std::hash::Hash,
{
    let mut sim = 0.0;
    for (k, av) in a.iter() {
        let bv = b.get(k).unwrap_or(&0.0);
        sim += bv * av;
    }

    let ma = a.iter().fold(0.0, |a, (_k, v)| a + v.powi(2));
    let mb = b.iter().fold(0.0, |a, (_k, v)| a + v.powi(2));
    sim.powi(2) / (ma * mb)
}




#[test]
fn test_similarity() {
    assert_eq!(1.0, similarity(&ENGLISH_FREQ, &ENGLISH_FREQ));
    // assert_eq!(1.0, similarity(a: &HashMap<T, f64>, b: &HashMap<T, f64>))

}
#[test]
fn s1_c3() {
    let mut res_vector = Vec::new();
    let b =
        from_hex("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736").unwrap();
    for n in 1..=255 {
        let decrypted: Vec<u8> = b.iter().map(|x| x ^ n).collect();
        let res = std::str::from_utf8(&decrypted);
        if res.is_err() {
            continue;
        }
        let english = res.unwrap();
        let mut freq = HashMap::new();
        let mut total = 0;
        for c in english.chars() {
            total += 1;
            let counter = freq.entry(c.to_ascii_uppercase()).or_insert(0.0);
            *counter += 1.0;
        }

        for (k, _v) in ENGLISH_FREQ.iter() {
            let counter = freq.entry(*k).or_insert(0.0);
            *counter = *counter / total as f64;
        }
        let sim = similarity(&ENGLISH_FREQ, &freq);
        res_vector.push((sim, english.to_owned()));
    }

    res_vector.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert!(true);
}

#[test]
fn s1_c4() {
    let mut res_vector = Vec::new();
    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines("./S1C4data.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(input) = line {
                let input = from_hex(&input).unwrap();
                let inner_res_vector = find_decode_xor(&input);
                
                let inner_res_vector: Vec<(f64, u8, String)> = inner_res_vector
                    .iter()
                    .map(|(f, k)| {
                        let b_english: Vec<u8> = input.iter().map(|x| x ^ k).collect();
                        let english = std::str::from_utf8(&b_english).unwrap();
                        (*f, *k, english.to_owned())
                }).collect();
                res_vector.extend(inner_res_vector);
            }
        }
    }
    res_vector.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    res_vector.reverse();
    for res in res_vector.iter() .take(50) {
        println!("{} {} {}", res.0, res.1, res.2)
    }
    assert!(true);
}

#[test]
fn s1_c5() {
    let input = "Burning 'em, if you ain't quick and nimble
I go crazy when I hear a cymbal"
        .as_bytes();
    let key = "ICE".as_bytes();
    let actual = to_hex(&repeat_xor(input, key)).unwrap();
    // Do I need the new line?
    let expected = "0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f";
    assert_eq!(expected, actual);
}

#[test]
fn test_repeating_xor() {
    let input = "Burning 'em, if you ain't quick and nimble
I go crazy when I hear a cymbal"
        .as_bytes();

    let key = "ICE".as_bytes();
    assert_eq!(input, repeat_xor(&repeat_xor(input, key), key))
}

