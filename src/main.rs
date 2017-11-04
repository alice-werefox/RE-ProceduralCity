extern crate obj;
extern crate noise;

use std::path::Path;

fn main() {
    let obj = obj::Obj::load("data/test.obj").unwrap();

    println!("Postiion: {}", obj.position);
}
