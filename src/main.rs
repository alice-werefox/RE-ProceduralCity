extern crate obj;
extern crate noise;

use std::io::Result;
use std::path::Path;
use obj::{Obj, SimplePolygon};

// A function called test that takes in 1 32-bit integer
// and returns a 32-bit integer.
/*
fn test(x: i32) -> i32 {
    // Yay conditions.
    if x > 4 {
        return x;
    }

    // Normally, we could do this to return.
    return x + 4;
    // Where we would need a semicolon because
    // we are passing the return to the return keyword
    // rather than just ending the function.

    // This doesn't need a semicolon because it will be
    // inplicitly returned since it is as the end of the
    // function definition.
    x + 4
}
*/

fn main() {

    let path = Path::new("data/teapot.obj");
    let maybe_obj: Result<Obj<SimplePolygon>> = Obj::load(&path);

    if let Ok(obj) = maybe_obj {
        println!("Postiion: {:?}", obj.position);
    }
    /*
    else if Err(error) = maybe_obj {

    }
    */
}
