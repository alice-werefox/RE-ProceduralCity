// REralCity
// Alex Huddleston / Jeremy Martin

extern crate obj;
extern crate noise;
extern crate cgmath;
extern crate rayon;

use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use obj::{Obj, SimplePolygon, IndexTuple};
use noise::Fbm;
//use noise::Seedable;
use noise::MultiFractal;
use noise::NoiseModule;
use cgmath::Vector3;
use cgmath::ElementWise;
use rayon::prelude::*;

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

fn distance_a_to_b(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    ((bx - ax) * (bx - ax) + (by - ay) * (by - ay)).sqrt()
}

fn return_at(x: f32, y: f32, fbmnoise: &Fbm<f32>) -> f32 {
    let zero_to_one_noise = (1.0 + fbmnoise.get([x, y])) * 0.5;
    let z = 0.0f32;
    let m = z.max(1.0 - 1.0 * distance_a_to_b(x, y, 0.5, 0.5));

    return (zero_to_one_noise * 0.90) + (m * 0.10);
}

fn find_l_w(obj: &Obj<SimplePolygon>) -> (f32, f32) {
    if let Some(first) = obj.position.first() {
        let initial = (first[0], first[1], first[0], first[1]);

        let min_maxes = obj.position.iter().fold(initial, |mut acc, point| {
            if acc.0 > point[0] {
                acc.0 = point[0];
            } else if acc.2 < point[0] {
                acc.2 = point[0];
            }

            if acc.1 > point[1] {
                acc.1 = point[1];
            } else if acc.3 < point[1] {
                acc.3 = point[1];
            }
            acc
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

fn duplicate<T>(
    positions: &[Vector3<f32>],
    translation: Vector3<f32>,
    height_scalar: f32,
    file: Arc<Mutex<BufWriter<T>>>,
) where
    T: Write,
{
    let v = Vec::new();
    let c = Cursor::new(v);
    let mut b = BufWriter::new(c);

    for point in positions {
        write!(
            b,
            "v  {} {} {}\n",
            point.x + translation.x,
            point.y + translation.y,
            point.z * height_scalar * 2.0
        ).unwrap();
    }

    file.lock()
        .unwrap()
        .write(b.into_inner().unwrap().into_inner().as_ref())
        .unwrap();
}

fn write_positions<T>(
    positions: &[Vector3<f32>],
    layers: i32,
    spacing: f32,
    length: f32,
    width: f32,
    file: Arc<Mutex<BufWriter<T>>>,
) where
    T: Write + Send,
{
    let length = length + spacing;
    let width = width + spacing;

    let fbm: Fbm<f32> = Fbm::new()
        .set_octaves(1)
        .set_frequency(6.0)
        .set_persistence(3.0)
        .set_lacunarity(30.0);

    duplicate(
        positions,
        Vector3::new(0.0, 0.0, 0.0),
        return_at(0.5, 0.5, &fbm),
        file.clone(),
    );

    (0..((layers * (layers - 1)) * 4))
        .into_par_iter()
        .for_each(move |current_duplicate| {
            let current_layer = (0.5 * (((current_duplicate + 1) as f32).sqrt() - 1.0)) as i32 + 1;
            let current_duplicate = current_duplicate - 4 * current_layer * (current_layer - 1);

            let current_ratio = current_duplicate as f32 / (current_layer as f32 * 8.0);

            let unit_translation = if current_ratio <= 1.0 / 4.0 {
                Vector3::new(1.0, -1.0 + (current_ratio * 8.0), 0.0)
            } else if current_ratio <= 2.0 / 4.0 {
                Vector3::new(1.0 - ((current_ratio) - 1.0 / 4.0) * 8.0, 1.0, 0.0)
            } else if current_ratio <= 3.0 / 4.0 {
                Vector3::new(-1.0, 1.0 - ((current_ratio) - 2.0 / 4.0) * 8.0, 0.0)
            } else {
                Vector3::new(-1.0 + ((current_ratio) - 3.0 / 4.0) * 8.0, -1.0, 0.0)
            };

            let translation = current_layer as f32 *
                Vector3::new(length * unit_translation.x, width * unit_translation.y, 0.0);

            // gets into range -1 to +1
            let coord =
                1.0 / 5.0 *
                    translation.mul_element_wise(
                        Vector3::new(1.0 / length as f32, 1.0 / width as f32, 0.0),
                    );

            // gets into range -0.4 to +0.4
            let coord = 0.4 * coord;

            // gets into range 0.1 to 0.9
            let coord = coord + Vector3::new(0.5, 0.5, 0.0);

            let height_scalar = return_at(coord.x, coord.y, &fbm);

            duplicate(&positions, translation, height_scalar, file.clone())
        })
}

fn write_faces<T>(
    faces: &[Vec<IndexTuple>],
    n_positions: usize,
    layers: usize,
    file: Arc<Mutex<BufWriter<T>>>,
) where
    T: Write + Send,
{
    (0..(2 * layers - 1).pow(2)).into_par_iter().for_each(
        move |current_value| {
            let v = Vec::new();
            let c = Cursor::new(v);
            let mut b = BufWriter::new(c);
            let offset = n_positions * current_value + 1;

            for current_face in faces {
                write!(b, "f").unwrap();
                for value in current_face {
                    write!(b, " {}/", value.0 + offset).unwrap();
                    if let Some(i) = value.1 {
                        write!(b, "{}", i + offset).unwrap();
                    }
                    write!(b, "/").unwrap();
                    if let Some(j) = value.2 {
                        write!(b, "{}", j + offset).unwrap();
                    }
                }
                write!(b, "\n").unwrap();
            }

            file.lock()
                .unwrap()
                .write(b.into_inner().unwrap().into_inner().as_ref())
                .unwrap();
        },
    );
}

fn main() {
    let path = Path::new("data/test.obj");
    let obj: Obj<SimplePolygon> = Obj::load(&path).expect("Failed to load input obj");

    let layers = 80;
    let spacing = 1.0;

    let (length, width) = find_l_w(&obj);

    let input_positions: Vec<_> = obj.position
        .iter()
        .map(|point| Vector3::new(point[0], point[1], point[2]))
        .collect();


    let filename = Path::new("target/noice.obj");
    let file_buf_writer = BufWriter::new(File::create(filename).unwrap());
    let file = Arc::new(Mutex::new(file_buf_writer));

    write_positions(
        &input_positions,
        layers,
        spacing,
        length,
        width,
        file.clone(),
    );
    write_faces(
        &obj.objects[0].groups[0].polys,
        obj.position.len(),
        layers as usize,
        file,
    );
}
