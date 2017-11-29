// REralCity
// Alex Huddleston / Jeremy Martin

extern crate obj;
extern crate noise;
extern crate cgmath;

use std::io::Result;
use std::path::Path;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use obj::{Obj, SimplePolygon, IndexTuple};
use noise::Fbm;
//use noise::Seedable;
//use noise::MultiFractal;
use noise::NoiseModule;
use cgmath::Vector3;
use cgmath::ElementWise;

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
    let n = 20.0 * fbmnoise.get([x, y]);
    let n = n - n.floor();

    let m = distance_a_to_b(x, y, 0.5, 0.5);

    return (m * 0.15) + (n * 0.85);
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

fn calculate_angle(current_duplicate: i32, current_layer: i32) -> f32 {
    (current_duplicate / (2 * current_layer)) as f32 * (0.5 * PI)
}

fn calculate_translation(length: f32, width: f32, angle: f32) -> Vector3<f32> {
    Vector3::new(length * angle.cos(), width * angle.sin(), 0.0)
}

fn duplicate(
    positions: Vec<Vector3<f32>>,
    translation: Vector3<f32>,
    height_scalar: f32,
) -> Vec<Vector3<f32>> {
    positions
        .iter()
        .map(|point| {
            Vector3::new(
                point.x + translation.x,
                point.y + translation.y,
                point.z * height_scalar,
            )
        })
        .collect()
}

fn generate_city(
    positions: Vec<Vector3<f32>>,
    layers: i32,
    spacing: f32,
    length: f32,
    width: f32,
) -> Vec<Vector3<f32>> {
    let length = length + spacing;
    let width = width + spacing;

    let mut temp = Vector3::new(0.0, 0.0, 0.0);

    let mut coord = Vector3::new(0.5, 0.5, 0.0);

    let fbm: Fbm<f32> = Fbm::new();

    (1..layers).fold(positions.clone(), |acc_positions, current_layer| {
        temp.x = -length * (current_layer as f32);
        temp.y = -width * (current_layer as f32);

        (0..(current_layer * 8)).fold(acc_positions, |mut acc_positions, current_duplicate| {

            let angle = calculate_angle(current_duplicate, current_layer);

            let translation = calculate_translation(length, width, angle);
            temp += translation;

            coord += translation.mul_element_wise(Vector3::new(
                (2.0 / (layers as f32 * 2.0 - 1.0)),
                (2.0 / (layers as f32 * 2.0 - 1.0)),
                0.0,
            ));

            let height_scalar = return_at(coord.x, coord.y, &fbm);

            acc_positions.extend(duplicate(positions.clone(), temp, height_scalar));

            acc_positions
        })
    })
}

fn copy_faces(
    faces: Vec<Vec<IndexTuple>>,
    n_positions: usize,
    layers: usize,
) -> Vec<Vec<IndexTuple>> {
    (0..(2 * layers - 1).pow(2)).fold(Vec::new(), |mut acc_faces, current_value| {
        let offset = n_positions * current_value + 1;

        acc_faces.extend(faces.iter().map(|current_face| {
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
        }));

        acc_faces
    })
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

        let input_positions = obj.position
            .iter()
            .map(|point| Vector3::new(point[0], point[1], point[2]))
            .collect();

        let output_positions = generate_city(input_positions, layers, spacing, length, width);

        println!("Objects: {:?}", obj.objects[0].groups[0].polys[0]);

        let output_faces = copy_faces(
            obj.objects[0].groups[0].polys.clone(),
            obj.position.len(),
            layers as usize,
        );

        save(Path::new("build/noice.obj"), output_positions, output_faces);
    }
}
