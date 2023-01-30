#![allow(non_snake_case)]

use bufferGeometry::BufferGeometry;
use cgmath::{MetricSpace, Vector3};
use perlin::Perlin;
use std::collections::VecDeque;

pub mod sphere;
pub mod scrape;
pub mod perlin;
pub mod bufferGeometry;

extern crate rand;

#[repr(C)]
pub struct RockObjParams {
    pub meshNoiseScale:f64,
    pub meshNoiseStrength:f64,
    pub scrapeCount:i32,
    pub scrapeMinDist:f64,
    pub scrapeStrength:f64,
    pub scrapeRadius:f64,
    pub scale:[f64;3],
    pub varyStrength:f64,
}

pub fn Rock(rock_obj_params:RockObjParams) -> BufferGeometry
{
    let (mut positions, indices, mut normals) = sphere::CreatSphere(1.0, 25, 25);

    // OPTIMIZATION: we are always using the same sphere as base for the rock,
    // so we only need to compute the adjacent positions once.
    let adjacentVertices = scrape::GetNeighbours(positions.len(), &indices);

    // 这段代码是在查找一些随机位置，并确保它们之间的距离不太近。
    // 它会随机选择一个位置，然后检查它与其他已选择的位置之间的距离是否小于某个最小值（rock_obj_params.scrapeMinDist）。
    // 如果是，则重新尝试找一个位置，否则就将它添加到选择的位置列表中。
    // 如果尝试次数超过了一定次数（100次），则无论它与其他位置的距离是否太近都将其添加到列表中，以避免无限循环。
    let mut scrapeIndices = Vec::with_capacity(rock_obj_params.scrapeCount as usize);
    for _ in 0..rock_obj_params.scrapeCount {
        let mut attempts = 0;

        // find random position which is not too close to the other positions.
        loop {
            let randIndex = (positions.len() as f64 * rand::random::<f64>()).floor() as usize;
            let p = positions[randIndex];

            let mut tooClose = false;
            // check that it is not too close to the other vertices.
            for j in 0..scrapeIndices.len() {
                let q = positions[scrapeIndices[j]];
                if p.distance2(q) < rock_obj_params.scrapeMinDist {
                    tooClose = true;
                    break;
                }
            }
            attempts=attempts+1;

            // if we have done too many attempts, we let it pass regardless.
            // otherwise, we risk an endless loop.
            if tooClose && attempts < 100 {
                continue;
            } else {
                scrapeIndices.push(randIndex);
                break;
            }
        }
    }

    let mut traversed = vec![false; positions.len()];
    let mut stack = VecDeque::with_capacity(100);
    // now we scrape at all the selected positions.
    for i in 0..scrapeIndices.len() {
        traversed.iter_mut().for_each(|x| *x = false);
        stack.clear();
        scrape::Main(scrapeIndices[i], &mut positions, &mut normals, &adjacentVertices, rock_obj_params.scrapeStrength, rock_obj_params.scrapeRadius, &mut traversed, &mut stack);
    }

    /*
     * Finally, we apply a Perlin noise to slightly distort the mesh and then scale the mesh.
     */
    let perlin = Perlin::new(rand::random());
    for i in 0..positions.len() {
        let p = positions[i];

        let noise = rock_obj_params.meshNoiseStrength * perlin.Noise(rock_obj_params.meshNoiseScale * p[0], rock_obj_params.meshNoiseScale * p[1], rock_obj_params.meshNoiseScale * p[2]);

        positions[i][0] += noise;
        positions[i][1] += noise;
        positions[i][2] += noise;

        positions[i][0] *= rock_obj_params.scale[0];
        positions[i][1] *= rock_obj_params.scale[1];
        positions[i][2] *= rock_obj_params.scale[2];

        positions[i]=positions[i]*3.0;

        positions[i][0] = ((positions[i][0] + std::f64::EPSILON) * 100.0).round() / 100.0;
        positions[i][1] = ((positions[i][1] + std::f64::EPSILON) * 100.0).round() / 100.0;
        positions[i][2] = ((positions[i][2] + std::f64::EPSILON) * 100.0).round() / 100.0;
    }

   
    BufferGeometry::new(positions, FlatU32(indices))
}

// fn FlatF64(date:Vec<Vector3<f64>>) -> Vec<f64> {
//     let mut flat = Vec::with_capacity(date.len()*3);
//     for i in 0..date.len() {
//         flat[i*3+0] = date[i][0];
//         flat[i*3+1] = date[i][1];
//         flat[i*3+2] = date[i][2];
//     }
//     flat
// }

fn FlatU32(date:Vec<Vector3<u32>>) -> Vec<u32> {
    let mut flat = vec![0; date.len()*3];
    for i in 0..date.len() {
        flat[i*3+0] = date[i][0];
        flat[i*3+1] = date[i][1];
        flat[i*3+2] = date[i][2];
    }
    flat
}