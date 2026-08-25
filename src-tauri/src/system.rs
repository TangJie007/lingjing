#[cfg(windows)]
pub fn detect_low_power_mode() -> bool {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    unsafe {
        let mut status: MEMORYSTATUSEX = std::mem::zeroed();
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut status).is_err() {
            return false;
        }
        // < 6 GB physical RAM → low power tier
        status.ullTotalPhys < 6 * 1024 * 1024 * 1024
    }
}

#[cfg(not(windows))]
pub fn detect_low_power_mode() -> bool {
    false
}

pub fn media_path_exists(uri: &str) -> bool {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("asset://")
        || trimmed.starts_with('/')
    {
        return true;
    }
    let path = trimmed
        .strip_prefix("file://")
        .unwrap_or(trimmed)
        .replace('/', std::path::MAIN_SEPARATOR_STR);
    std::path::Path::new(&path).is_file()
}
