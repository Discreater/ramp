use std::time::Instant;

use ramp::{int::mtgy::MtgyModulus, Int};

fn main() {
    let a = Int::from_str_radix("4cf98e54ab14095eccfde5bec1255f69f57a7e6ee86cf3e670c871c9aa8c3c3a15ad65ffdb8ea85f1c585e862426e3911f017c438a72ec8fe6c989d96382ae032fada0f14d50db28922f88059c3d070934a916ec8e9f8d7d13e682b9e662513d22576c826f183a07e9f9da5925dc08e301870cc357c5addb6c723e9003a77179", 16).unwrap();
    let b = Int::from_str_radix("3cf98e54ab14095eccfde5bec1255f69f57a7e6ee86cf3e670c871c9aa8c3c3a15ad65ffdb8ea85f1c585e862426e3911f017c438a72ec8fe6c989d96382ae032fada0f14d50db28922f88059c3d070934a916ec8e9f8d7d13e682b9e662513d22576c826f183a07e9f9da5925dc08e301870cc357c5addb6c723e9003a77179", 16).unwrap();
    let m = Int::from_str_radix("8cf98e54ab14095eccfde5bec1255f69f57a7e6ee86cf3e670c871c9aa8c3c3a15ad65ffdb8ea85f1c585e862426e3911f017c438a72ec8fe6c989d96382ae032fada0f14d50db28922f88059c3d070934a916ec8e9f8d7d13e682b9e662513d22576c826f183a07e9f9da5925dc08e301870cc357c5addb6c723e9003a771798cf98e54ab14095eccfde5bec1255f69f57a7e6ee86cf3e670c871c9aa8c3c3a15ad65ffdb8ea85f1c585e862426e3911f017c438a72ec8fe6c989d96382ae032fada0f14d50db28922f88059c3d070934a916ec8e9f8d7d13e682b9e662513d22576c826f183a07e9f9da5925dc08e301870cc357c5addb6c723e9003a77179", 16).unwrap();
    let mg = MtgyModulus::new(&m);
    let mut a_bar = mg.to_mtgy(&a);
    let b_bar = mg.to_mtgy(&b);
    let iters = 1000000;

    let start = Instant::now();
    for _ in 0..iters {
        a_bar = mg.mul(&a_bar, &b_bar);
    }
    let duration = start.elapsed();

    let res = mg.to_int(&a_bar);
    println!("{}", res);
    println!("Time elapsed in {} iter is: {:?}", iters, duration);
    println!("Average time: {:?}", duration / iters);
}
