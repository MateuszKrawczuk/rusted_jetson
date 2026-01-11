/// Get number of CPU cores
pub fn get_core_count() -> usize {
    let path = Path::new("/proc/cpuinfo");
    if let Ok(content) = fs::read_to_string(path) {
        content.lines()
            .filter(|line| line.starts_with("processor"))
            .count()
    } else {
        // Fallback to environment
        num_cpus::get()
    }
}
