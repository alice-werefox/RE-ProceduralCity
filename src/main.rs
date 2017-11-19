// REralCity
// Alex Huddleston / Jeremy Martin

extern crate obj;
extern crate noise;
extern crate cgmath;
extern crate rayon;

use std::io::Result;
use std::path::Path;
use std::fs::File;
use std::io::Write;
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

fn duplicate(
    positions: &[Vector3<f32>],
    translation: Vector3<f32>,
    height_scalar: f32,
) -> Vec<Vector3<f32>> {
    positions
        .par_iter()
        .map(|point| {
            Vector3::new(
                point.x + translation.x,
                point.y + translation.y,
                point.z * height_scalar * 2.0,
            )
        })
        .collect()
}

fn generate_city(
    positions: &[Vector3<f32>],
    layers: i32,
    spacing: f32,
    length: f32,
    width: f32,
) -> Vec<Vector3<f32>> {
    let length = length + spacing;
    let width = width + spacing;

    let fbm: Fbm<f32> = Fbm::new()
        .set_octaves(1)
        .set_frequency(6.0)
        .set_persistence(3.0)
        .set_lacunarity(30.0);

    let mut output_positions = duplicate(
        positions,
        Vector3::new(0.0, 0.0, 0.0),
        return_at(0.5, 0.5, &fbm),
    );

    let rest_vec: Vec<_> = (1..layers)
        // .into_par_iter()
        .flat_map(|current_layer| {
            (0..(current_layer * 8))
                // .into_par_iter()
                .flat_map(|current_duplicate| {
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
                        1.0/layers as f32 * translation.mul_element_wise(
                            Vector3::new(1.0 / length as f32, 1.0 / width as f32, 0.0),
                        );

                    // gets into range -0.4 to +0.4
                    let coord = 0.4 * coord;

                    // gets into range 0.1 to 0.9
                    let coord = coord + Vector3::new(0.5, 0.5, 0.0);

                    let height_scalar = return_at(coord.x, coord.y, &fbm);

                    duplicate(&positions, translation, height_scalar)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    output_positions.extend(rest_vec);

    output_positions
}

fn copy_faces(
    faces: &[Vec<IndexTuple>],
    n_positions: usize,
    layers: usize,
) -> Vec<Vec<IndexTuple>> {
    (0..(2 * layers - 1).pow(2))
        .into_par_iter()
        .flat_map(|current_value| {
            let offset = n_positions * current_value + 1;

            faces
                .par_iter()
                .map(|current_face| {
                    current_face
                        .iter()
                        .map(|index_tuple| {
                            IndexTuple(
                                index_tuple.0 + offset,
                                index_tuple.1.map(|i| i + offset),
                                index_tuple.2.map(|j| j + offset),
                            )
                        })
                        .collect()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn save(filename: &Path, positions: Vec<Vector3<f32>>, faces: Vec<Vec<IndexTuple>>) {
    let mut file = File::create(filename).unwrap();

    for pos in positions {
        write!(file, "v  {} {} {}\n", pos[0], pos[1], pos[2]).unwrap();
    }

    for face in faces {
        write!(file, "f").unwrap();
        for value in face {
            write!(file, " {}/", value.0).unwrap();
            if let Some(i) = value.1 {
                write!(file, "{}", i).unwrap();
            }
            write!(file, "/").unwrap();
            if let Some(j) = value.2 {
                write!(file, "{}", j).unwrap();
            }
        }
        write!(file, "\n").unwrap();
    }
}

fn main() {

    let path = Path::new("data/test.obj");
    let maybe_obj: Result<Obj<SimplePolygon>> = Obj::load(&path);

    if let Ok(obj) = maybe_obj {
        println!("Position: {:?}", obj.position);

        let layers = 10;
        let spacing = 1.0;

        let (length, width) = find_l_w(&obj);

        println!("Length: {} Width: {}", length, width);

        let input_positions: Vec<_> = obj.position
            .iter()
            .map(|point| Vector3::new(point[0], point[1], point[2]))
            .collect();

        let output_positions = generate_city(&input_positions, layers, spacing, length, width);

        println!("Objects: {:?}", obj.objects[0].groups[0].polys[0]);

        let output_faces = copy_faces(
            &obj.objects[0].groups[0].polys,
            obj.position.len(),
            layers as usize,
        );

        save(
            Path::new("target/noice.obj"),
            output_positions,
            output_faces,
        );
    }
}
