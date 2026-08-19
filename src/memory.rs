use std::os::raw::c_void;

#[derive(Debug, PartialEq)]
pub enum MemoryPressure {
    Normal,
    Warning,
    Critical,
    Unknown,
}

impl MemoryPressure {
    /// Returns the icon's raw PNG bytes, embedded into the binary at
    /// compile time via include_bytes!.
    ///
    /// This used to return a runtime path ("assets/icons/...") that
    /// load_icon() opened with image::open(). That worked under `cargo run`
    /// (CWD = project root) but broke in the bundled .app: Finder launches
    /// it with a different working directory, and `cargo bundle` doesn't
    /// copy assets/ into the bundle in the first place, so the path never
    /// resolved and the app panicked before the tray icon was ever created —
    /// silently, since a GUI app launched from Finder has no terminal to
    /// print the panic to. Embedding sidesteps both problems: there's no
    /// path to resolve and nothing extra to bundle.
    pub fn icon_bytes(&self) -> &'static [u8] {
        match self {
            MemoryPressure::Normal => include_bytes!("../assets/icons/memory-chip-green.png"),
            MemoryPressure::Warning => include_bytes!("../assets/icons/memory-chip-yellow.png"),
            MemoryPressure::Critical => include_bytes!("../assets/icons/memory-chip-red.png"),
            MemoryPressure::Unknown => include_bytes!("../assets/icons/memory-chip-yellow.png"),
        }
    }
}
#[derive(Debug)]
pub struct SystemMemoryInfo {
    pub pressure_level: MemoryPressure,
    pub total_memory_gb: f64,
    pub used_memory: f64,
}

pub fn get_sysctl_i32(name: &str) -> Option<i32> {
    let c_name = std::ffi::CString::new(name).ok()?;
    let mut val: i32 = 0;
    let mut size = std::mem::size_of::<i32>() as libc::size_t;

    unsafe {
        if libc::sysctlbyname(
            c_name.as_ptr(),
            &mut val as *mut i32 as *mut c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) == 0
        {
            Some(val)
        } else {
            None
        }
    }
}

fn get_sysctl_u64(name: &str) -> Option<u64> {
    let c_name = std::ffi::CString::new(name).ok()?;
    let mut val: u64 = 0;
    let mut size = std::mem::size_of::<u64>() as libc::size_t;

    unsafe {
        if libc::sysctlbyname(
            c_name.as_ptr(),
            &mut val as *mut u64 as *mut c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) == 0
        {
            Some(val)
        } else {
            None
        }
    }
}

pub fn poll_memory() -> Option<SystemMemoryInfo> {
    let total_bytes = get_sysctl_u64("hw.memsize").unwrap_or(0);
    let total_memory_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    // fetch mem pressure
    let pressure_val = get_sysctl_i32("vm.memory_pressure").unwrap_or(0);

    let pressure_level = match pressure_val {
        0..=30 => MemoryPressure::Normal,
        31..=70 => MemoryPressure::Warning,
        _ => MemoryPressure::Critical,
    };

    let page_size = get_sysctl_i32("vm.pagesize").unwrap_or(4096) as u64;
    let free_pages = get_sysctl_i32("vm.page_free_count").unwrap_or(0) as u64;
    let free_bytes = free_pages * page_size;
    let used_bytes = total_bytes.saturating_sub(free_bytes);
    let used_memory = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    Some(SystemMemoryInfo {
        pressure_level,
        total_memory_gb,
        used_memory,
    })
}
