#[macro_export]
macro_rules! log_info {
    ($($tt:tt)*) => ({
        use colored::*;
        println!("{} {}", "Info:".green(), format!($($tt)*));
    })
}

#[macro_export]
macro_rules! log_error {
    ($($tt:tt)*) => ({
        use colored::*;
        eprintln!("{} {}", "Error:".red(), format!($($tt)*));
    })
}