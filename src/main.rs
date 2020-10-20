use std::error::Error;
use std::{fmt::Write, num::ParseIntError};

extern crate base64;

use base64::encode;
use std::collections::HashMap;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

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
fn main() {}
// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn repeat_xor(input: &[u8], key: &[u8]) -> Vec<u8> {
    // This seems somewhat cryptic
    input
        .chunks(key.len())
        .map(|chunk| chunk.iter().zip(key).map(|(b, k)| b ^ k))
        .fold(Vec::new(), |mut acc, chunk| {
            chunk.for_each(|b| acc.push(b));
            acc
        })
}

pub fn find_decode_xor(input: &str) -> Vec<(f64, String)> {
    let b = decode_hex(input).unwrap();

    let mut res_vector = Vec::new();

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
    res_vector
}

pub fn hex_to_base64(hex_string: &str) -> Result<String, ParseIntError> {
    let bytes = decode_hex(hex_string)?;
    Ok(encode(bytes))
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b)?;
    }
    Ok(s)
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
fn s1_c1() {
    let actual = hex_to_base64("49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d");
    assert_eq!(
        Ok("SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t".to_owned()),
        actual
    );
}

#[test]
fn s1_c2() {
    let a = decode_hex("1c0111001f010100061a024b53535009181c").unwrap();
    let b = decode_hex("686974207468652062756c6c277320657965").unwrap();
    let c: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
    assert_eq!(
        "746865206b696420646f6e277420706c6179".to_owned(),
        encode_hex(&c).unwrap()
    )
}

#[test]
fn test_similarity() {
    assert_eq!(1.0, similarity(&ENGLISH_FREQ, &ENGLISH_FREQ));
}
#[test]
fn s1_c3() {
    let mut res_vector = Vec::new();
    let b =
        decode_hex("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736").unwrap();
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
    for t in res_vector {
        println!("{} {}", t.0, t.1);
    }

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
                println!("{}\n\n", input);

                let inner_res_vector = find_decode_xor(&input);
                res_vector.extend(inner_res_vector);
            }
        }
    }
    res_vector.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for res in res_vector.iter() {
        println!("{} {}", res.0, res.1)
    }
    assert!(true);
}

#[test]
fn s1_c5() {
    let input = "Burning 'em, if you ain't quick and nimble
I go crazy when I hear a cymbal"
        .as_bytes();
    let key = "ICE".as_bytes();
    let actual = encode_hex(&repeat_xor(input, key)).unwrap();
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
