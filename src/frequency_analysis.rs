// Gives the hamming distance between bits of a and b
pub fn hamming(a: &[u8], b: &[u8]) -> u64 {
    let d = a.iter().zip(b.iter()).fold(0, |mut acc, (aa, bb)| {
        for i in 0..8 {
            let bit_a = aa >> i;
            let bit_b = bb >> i;
            acc += (bit_a ^ bit_b) & 1;
        }
        acc
    });
    let len_difference = if a.len() > b.len() {
        a.len() - b.len()
    } else {
        b.len() - a.len()
    };
    (d + len_difference as u8).into()
}

#[test]
fn test_hamming() {
    assert_eq!(
        37,
        hamming("this is a test".as_bytes(), "wokka wokka!!!".as_bytes())
    );
}
