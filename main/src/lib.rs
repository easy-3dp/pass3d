use libc::{c_void, c_int, memcpy};

#[no_mangle]
pub unsafe extern fn p3d_process(_rock_obj_params:rock_obj::RockObjParams, _pre:*mut c_void, function:c_int, _out_hash:*mut c_void, _out_poisitons:*mut c_void, _out_indices:*mut c_void, _out_normals:*mut c_void) -> i32 {

	let mut geo = rock_obj::Rock(_rock_obj_params);

	geo.wavefront_loadobj();
	geo.ComputeVertexNormals();

	let data_str = geo.parse();
	let data = data_str.as_bytes();

	let pre:[u8; 4] = [0, 0, 0, 0];
	memcpy(pre.as_ptr() as *mut c_void, _pre, 4);

	let res = p3d::p3d_process(data, p3d::AlgoType::Grid2dV2, 8, 12, Some(pre));
	
	if let Ok(v) = res {
		if v.len() > 0 {
			memcpy(_out_poisitons, geo.positions.as_ptr() as *const c_void, 14448);
			memcpy(_out_indices,   geo.indices  .as_ptr() as *const c_void, 14400);
			memcpy(_out_normals,   geo.normals  .as_ptr() as *const c_void, 14448);
			memcpy(_out_hash, v[0].as_ptr() as *const c_void, 64);
			1
		} else {
			0
		}
	} else {
		0
	}

}

#[no_mangle]
pub unsafe extern fn get_version(str:*mut c_void, function:c_int) -> i32 {
	let bytes = "Official algorithm 0.6.2 (Grid2d_v2)".as_bytes();
	memcpy(str, bytes.as_ptr() as *const c_void, bytes.len());
	bytes.len() as i32
}