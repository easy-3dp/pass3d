use libc::{c_void, memcpy};

use std::convert::TryInto;
use std::f32::consts::PI;
use std::slice;
use cgmath::{InnerSpace, Matrix4, Rad, Vector3, Vector4, VectorSpace};
use genmesh::{MapVertex, Triangle, Triangulate};
use genmesh::generators::{IndexedPolygon, SharedVertex, SphereUv};
use rand::prelude::*;
use ecies_ed25519::encrypt;
use schnorrkel::{ExpansionMode, MiniSecretKey};

#[no_mangle]
pub unsafe extern fn p3d_process(_pre:*mut c_void, _out_hash:*mut c_void, _out_str:*mut c_void, _out_len:*mut i32) -> i32 {

	let data = create_mining_obj();

	let pre:[u8; 4] = [0, 0, 0, 0];
	memcpy(pre.as_ptr() as *mut c_void, _pre, 4);

	let res = p3d::p3d_process(data.as_ref(), p3d::AlgoType::Grid2dV3, 8, 12, Some(pre));
	
	if let Ok(v) = res {
		if v.len() > 0 && v[0] != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" {
            memcpy(_out_hash, v[0].as_ptr() as *const c_void, 64);
			memcpy(_out_str, data.as_ptr() as *const c_void, data.len());
            *_out_len = data.len() as i32;
			1
		} else {
			0
		}
	} else {
		0
	}

}

#[no_mangle]
pub unsafe extern fn get_version(str:*mut c_void) -> i32 {
	let bytes = "Official algorithm 0.6.3".as_bytes();
	memcpy(str, bytes.as_ptr() as *const c_void, bytes.len());
	bytes.len() as i32
}

#[no_mangle]
pub unsafe extern fn get_algorithm(str:*mut c_void) -> i32 {
	let bytes = "Grid2dV3.1".as_bytes();
	memcpy(str, bytes.as_ptr() as *const c_void, bytes.len());
	bytes.len() as i32
}

pub fn create_mining_obj() -> Vec<u8> {
    let dents_count = 36;
    let dent_size: f32 = 0.2;

    let object = SphereUv::new(13, 13);

    let mut vertices: Vec<Vector3<f32>> = object.shared_vertex_iter()
        .map(|v| v.pos.into())
        .map(|v: [f32; 3]| Vector3::new(v[0], v[1], v[2]))
        .collect();

    let mut rng = thread_rng();
    let vertices_count = vertices.len();
    for _ in 0..dents_count {
        let index = rng.gen_range(0, vertices_count);
        let distance = rng.gen_range(0.0, dent_size);
        vertices[index] = vertices[index].lerp(Vector3::new(0.0, 0.0, 0.0), distance);
    }

    let transformation_matrix = Matrix4::from_nonuniform_scale(0.8, 0.8, 1.0) *
        Matrix4::from_angle_x(Rad(PI / 2.0));

    vertices.iter_mut()
        .for_each(|v| {
            let v4 = Vector4::new(v.x, v.y, v.z, 1.0); // Convert to Vector4
            let transformed_v4 = transformation_matrix * v4;
            *v = Vector3::new(transformed_v4.x, transformed_v4.y, transformed_v4.z);
        });

    let triangles: Vec<Triangle<usize>> = object.indexed_polygon_iter()
        .triangulate()
        .collect();

    let mut obj_data = String::with_capacity(vertices_count * 100);

    obj_data.push_str("o\n");

    for vertex in vertices.iter() {
        obj_data.push_str(&format!("v {:.2} {:.2} {:.2}\n", vertex.x, vertex.y, vertex.z));
    }

    for vertex in vertices.iter() {
        let normal = vertex.normalize();
        obj_data.push_str(&format!("vn {:.4} {:.4} {:.4}\n", normal.x, normal.y, normal.z));
    }

    for triangle in triangles.iter() {
        let f = triangle.map_vertex(|i| i + 1);
        obj_data.push_str(&format!("f {}//{} {}//{} {}//{}\n", f.x, f.x, f.y, f.y, f.z, f.z));
    }

    obj_data.into_bytes()
}

#[no_mangle]
pub unsafe extern fn sign(_message:*mut c_void, _message_len:i32, _pub_key:*mut c_void, _hash:*mut c_void, _key:*mut c_void, _out_encrypted:*mut c_void, _out_encrypted_len:*mut i32, _out_sign:*mut c_void) -> i32 {

    let message = slice::from_raw_parts(_message as *const u8, _message_len as usize);

    let pub_key = slice::from_raw_parts(_pub_key as *const u8, 32);
    let pub_key = ecies_ed25519::PublicKey::from_bytes(&pub_key).unwrap();

    let hash = slice::from_raw_parts(_hash as *const u8, 32);
    let mut csprng = StdRng::from_seed(hash.try_into().unwrap());
    let encrypted = encrypt(&pub_key, message, &mut csprng).unwrap();

    let key = slice::from_raw_parts(_key as *const u8, 32);
    let key = MiniSecretKey::from_bytes(&key);
    if let Ok(key) = key {
        let key = key.expand(ExpansionMode::Ed25519);

        const CTX: &[u8] = b"Mining pool";
        let sign = key.sign_simple(CTX, &encrypted, &key.to_public());
        
        memcpy(_out_encrypted, encrypted.as_ptr() as *const c_void, encrypted.len());
        *_out_encrypted_len = encrypted.len() as i32;

        let sign = sign.to_bytes();
        memcpy(_out_sign, sign.as_ptr() as *const c_void, sign.len());

        return  1;
    } else {
        return  -1;
    }
}
