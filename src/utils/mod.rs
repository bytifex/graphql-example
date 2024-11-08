#[macro_export]
macro_rules! log_location {
    () => {
        format!("{}:{}", file!(), line!())
    };
}
