// REralCity
// Alex Huddleston / Jeremy Martin

extern crate obj;
extern crate noise;
extern crate cgmath;

use std::io::Result;
use std::path::Path;
use obj::{Obj, SimplePolygon};
use std::f64::consts::PI;
use noise::Fbm;
use noise::Seedable;
use noise::MultiFractal;
use noise::NoiseModule;

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

fn distance_a_to_b(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ((bx - ax)*(bx - ax) + (by - ay)*(by - ay)).sqrt()
}

fn noise_map(seed: usize, oct: usize, freq: f64, lacu: f64, pers: f64) -> Fbm<f64> {
    Fbm::new()
        .set_seed(seed)
        .set_octaves(oct)
        .set_frequency(freq)
        .set_lacunarity(lacu)
        .set_persistence(pers)
}

fn return_at(x: f64, y: f64, fbmnoise: &Fbm<f64>) -> f64 {
    let n = 20.0 * fbmnoise.get([x, y]);
    let n = n - n.floor();
    
    let m = distance_a_to_b(x, y, 0.5, 0.5);

    return (m*0.15) + (n*0.85);
}

fn find_l_w(obj: &Obj<SimplePolygon>) -> (f32, f32) {
    if let Some(first) = obj.position.first() {
        let initial = (
            first[0],
            first[1],
            first[0],
            first[1]
        );

        let min_maxes = obj.position.iter().fold(initial, |acc, point| {
            let acc = if acc.0 > point[0] {
                (point[0], acc.1, acc.2, acc.3)
            } else if acc.2 < point[0] {
                (acc.0, acc.1, point[0], acc.3)
            } else {
                acc
            };

            if acc.1 > point[1] {
                (acc.0, point[1], acc.2, acc.3)
            } else if acc.3 < point[1] {
                (acc.0, acc.1, acc.2, point[1])
            } else {
                acc
            }
        });

        (min_maxes.2 - min_maxes.0, min_maxes.3 - min_maxes.1)
    } else {
        (0.0, 0.0)
    }
}

/*
 * The current layer is how many iterations you are from the center,
 * the count is how far around the square you've gone on the current layer.
 * This outputs the angle at which to place the new duplicate relative
 * to the initial input obj's position.
*/

fn calculate_angle(count: i32, current_layer: i32) -> f64 {
    ((count as f64)/(2.0*(current_layer as f64)))*(0.5*PI)
}

fn generate_city(positions: Vec<[f32; 3]>, layers: i32, spacing: f32, length: f32, width: f32) -> Vec<[f32; 3]> {
    positions
}

fn main() {

    let path = Path::new("data/teapot.obj");
    let maybe_obj: Result<Obj<SimplePolygon>> = Obj::load(&path);

    if let Ok(obj) = maybe_obj {
        println!("Position: {:?}", obj.position);
    }
    /*
    else if Err(error) = maybe_obj {

    }
    */
}
