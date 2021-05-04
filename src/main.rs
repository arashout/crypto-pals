extern crate base64;

mod set1;


fn main() {
    let x = set1::fingerprint("something big");
    println!("{:?}",x);
}
