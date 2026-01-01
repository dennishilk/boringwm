pub fn init() {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .init();
}
