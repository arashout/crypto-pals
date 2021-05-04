extern crate base64;
extern crate hex;

use std::u8;
use std::fs;

const ASCII_SIZE: usize = 256;
type Similarity = (f64, String);
type Fingerprint = [f64; ASCII_SIZE];

pub fn hex_to_base64(hex_string: String) -> String {
    // Make vector of bytes from octets
    let mut bytes = Vec::new();
    for i in 0..(hex_string.len() / 2) {
        let res = u8::from_str_radix(&hex_string[2 * i..2 * i + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(e) => println!("Problem with hex: {}", e),
        };
    }

    base64::encode(&bytes) // now convert from Vec<u8> to b64-encoded String
}

pub fn hex_decode<'a>(hex: &'a str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for i in 0..(hex.len() / 2) {
        let res = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(e) => println!("Problem with hex: {}", e),
        };
    }
    bytes
}

pub fn fingerprint<'a>(s: &'a str) -> Fingerprint {
    let letters: Vec<usize> = s
        .chars()
        .map(|c| c as usize )
        .collect();
    let count = letters.len();
    let mut freq: [usize;ASCII_SIZE] = [0; ASCII_SIZE];
    letters.iter().filter(|x| **x < ASCII_SIZE).for_each(|n| {
        freq[*n] += 1;
    });
    let mut fp: Fingerprint = [0.0; ASCII_SIZE];
    freq.iter().enumerate().for_each(|(i, v)| {
        fp[i] = (*v as f64) / (count as f64);
    });

    let m = dot(&fp, &fp).sqrt();
    scale(&fp, 1.0 / m)
}

pub fn dot(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let mut product = 0.0;
    for i in 0..ASCII_SIZE {
        product += a[i] * b[i];
    }

    product
}

pub fn scale(a: &Fingerprint, m: f64) -> Fingerprint {
    let mut b: Fingerprint = [0.0; ASCII_SIZE];
    for i in 0..a.len() {
        b[i] = a[i] * m;
    }
    b
}

pub fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    let mut c = Vec::with_capacity(a.len());
    for i in 0..a.len() {
        c.push(a[i] ^ b[i]);
    }
    c
}

pub fn xor_single(a: &[u8], x: u8) -> Vec<u8> {
    let mut c = Vec::with_capacity(a.len());
    for i in 0..a.len() {
        c.push(a[i] ^ x);
    }
    c
}

pub fn decrypt_single_char_cipher(original: &[u8], fp: &Fingerprint) -> Option<Similarity> {
    let mut results: Vec<Similarity> = Vec::new();
    for x in 0..ASCII_SIZE {
        let d = xor_single(original, x as u8);
        if let Ok(s) = std::str::from_utf8(&d) {
            let similarity =  dot(fp, &fingerprint(s));
            if !similarity.is_nan(){
                results.push( (similarity, s.to_owned()) );
            }
        }
    }

    results.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("partial cmp failed!"));
    results.last().cloned()
}

pub fn encrypt_xor<'a, 'b>(plaintext: &'a str, key: &'b str) -> Vec<u8> {
    let key_bytes = key.as_bytes();
    let key_length = key_bytes.len();
    let plain_bytes = plaintext.as_bytes();


    let mut cipher_bytes = Vec::new();
    // TODO: Use chunks to try an parallelize
    let mut i = 0;
    while i < plain_bytes.len(){
        // If new line, just print out a newline?
        if plain_bytes[i] == 10 {

        }
        cipher_bytes.push(plain_bytes[i] ^ key_bytes[i % key_length] );
        i += 1;
    }

    cipher_bytes
}

macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !($x - $y < $d || $y - $x < $d) {
            panic!();
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hex_to_base64() {
        let hex_string = "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d";
        assert_eq!(
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t".to_string(),
            hex_to_base64(hex_string.to_string())
        );
    }

    #[test]
    fn test_hex_xor() {
        let x = hex_decode("1c0111001f010100061a024b53535009181c");
        let y = hex_decode("686974207468652062756c6c277320657965");
        let z: Vec<u8> = xor(&x, &y);
        assert_eq!("746865206b696420646f6e277420706c6179", hex::encode(&z));
    }

    #[test]
    fn test_fingerprint() {
        let x = fingerprint("something big");
        assert_delta!(1.0, dot(&x, &x), 0.001);
    }

    #[test]
    fn test_closest() {
        let x = fingerprint("something big");
        let y = fingerprint("something bigger");
        let z = fingerprint("something at alll");

        assert_delta!(0.95, dot(&x, &y), 0.1);
        assert_delta!(0.6, dot(&x, &z), 0.1);
    }

    #[test]
    fn test_decrypt_single() {
        let original =
            hex_decode("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736");

        let sp = fs::read_to_string("data/t8.shakespeare.txt").expect("Could not read shakespear");
        let fp = fingerprint(&sp);

        let res = decrypt_single_char_cipher(&original, &fp);
        assert_eq!("Cooking MC's like a pound of bacon", &res.unwrap().1);
    }

    #[test]
    fn test_plaintext_in_file(){
        let sp = fs::read_to_string("data/t8.shakespeare.txt").expect("Could not read shakespear");
        let fp = fingerprint(&sp);
        let lines: Vec<String> = fs::read_to_string("data/4.txt").expect("Could not read challenge 4").split("\n").map(|s| s.to_owned()).collect();
        let mut results = Vec::new();
        for line in lines {
            results.push(decrypt_single_char_cipher(&hex_decode(&line), &fp));
        }
        let mut valid: Vec<Similarity> = results.iter().filter(|x|x.is_some()).map(|x| x.clone().unwrap()).collect();
        valid.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("partial cmp failed!"));
        assert_eq!("Now that the party is jumping\n", valid.last().unwrap().1);
    }

    #[test]
    fn test_encrypt_repeating_xor(){
        let pt = "Burning 'em, if you ain't quick and nimble
I go crazy when I hear a cymbal";
        let res =  encrypt_xor(pt, "ICE");
        assert_eq!("0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f", hex::encode(res));
    }
}
