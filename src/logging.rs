#[macro_export]
macro_rules! log_info {
    ($($tt:tt)*) => ({
        println!($($tt)*);
    })
}

#[macro_export]
macro_rules! log_error {
    ($($tt:tt)*) => ({
        use colored::*;
        eprintln!("{} {}", "Error:".red(), format!($($tt)*));
    })
}