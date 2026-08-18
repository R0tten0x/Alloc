use std::ffi::CString;
use std::mem;

pub enum MemmoryPressure {
    Good,
    High,
    Critical,
}

pub fn poll_memory() -> Option<i32> {
    let name = CString::new("vm.memory_pressure").ok()?;
    let mut val: i32 = 0;
    let mut size = mem::size_of::<i32>() as libc::size_t;

    unsafe {
        let ret = libc::sysctlbyname(
            name.as_ptr(),
            (&mut val as *mut i32) as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );

        if ret == 0 { Some(val) } else { None }
    }
}
