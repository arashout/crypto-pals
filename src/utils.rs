use std::{fmt::Write, num::ParseIntError};
use std::error::Error;

use base64;

pub fn from_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn to_hex(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b)?;
    }
    Ok(s)
}

pub fn xor(a: &[u8], b: &[u8]) -> Vec<u8>{
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

pub fn hex_to_base64(hex_string: &str) -> Result<String, ParseIntError> {
    let bytes = from_hex(hex_string)?;
    Ok(base64::encode(bytes))
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
    let a = from_hex("1c0111001f010100061a024b53535009181c").unwrap();
    let b = from_hex("686974207468652062756c6c277320657965").unwrap();
    assert_eq!(
        "746865206b696420646f6e277420706c6179".to_owned(),
        to_hex(&xor(&a, &b)).unwrap()
    )
}


