impl PowerStats {
    pub fn get() -> Self {
        let mut stats = PowerStats::default();

        let i2c_path = Path::new("/sys/bus/i2c/devices");

        if !i2c_path.exists() {
            return PowerStats::default();
        }

        stats.rails = read_power_rails(&i2c_path);

        stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>() / 1000.0;

        stats
    }
}
