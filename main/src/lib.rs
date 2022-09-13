use libc::memset;
use libc::{c_void, c_char, memcpy};
use std::convert::TryInto;
use std::ffi::CStr;
use std::panic;

#[no_mangle]
pub unsafe extern fn gethash(s:*const c_char, pre:*const c_char, out:*mut c_void) -> i32 {

	if let Ok(data) = CStr::from_ptr(s).to_str() {
		let data = data.as_bytes();
		let _pre;
		if pre.is_null() {
			_pre = Option::None;
		}else{
			_pre = CStr::from_ptr(pre).to_bytes()[0..4].try_into().ok();
		}

		let result = panic::catch_unwind(||{
			p3d::p3d_process(data, p3d::AlgoType::Grid2d, 8, 66, _pre)
		});
		if let Ok(res) = result {
			if let Ok(v) = res {
				let len = v.len() as i32;
				memset(out, 0, 640);
				let mut address = out;
				for d in v {
					let tmp = d.as_ptr() as *const c_void;
					memcpy(address, tmp, 64);
					address = address.add(64);
				}
				return len;
			}
		}
	}
	
	-1

}