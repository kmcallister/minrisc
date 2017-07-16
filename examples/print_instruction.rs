extern crate minrisc;

use std::env;

fn main() {
    let num = u32::from_str_radix(
        &env::args().nth(1).unwrap(),
        16
    ).unwrap();

    println!("{:?}",
        minrisc::decode::decode(num));
}
