#[macro_export]
macro_rules! progress {
    ($name:tt, $($arg:tt)*) => {{
        ::std::eprint!("{:>12} ", $name.green().bold());
        ::std::eprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        ::std::eprint!("{}: ", "warning".yellow().bold());
        ::std::eprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        ::std::eprint!("{}: ", "error".red().bold());
        ::std::eprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! exit_with_error {
    ($($arg:tt)*) => {{
        error!($($arg)*);
        ::std::process::exit(1);
    }};
}
