

/// This module contains the `show` module.
pub mod show {
    #[macro_export]
    /// The `show` macro is used to print debug information with timestamp, file path, line number, and arguments.
    ///
    /// # Arguments
    ///
    /// * `$file` - The file path where the `show` macro is called.
    /// * `$line` - The line number where the `show` macro is called.
    /// * `$args` - The arguments to be printed.
    ///
    /// # Example
    ///
    /// ```
    /// show!("main.rs", 10, "Hello, world!", 42);
    /// you can also use the `file!()` and `line!()` macros to get the file path and line number where the `show` macro is called.
    /// show!(file!(), line!(), "Hello, world!", 42);
    /// ```
    ///
    /// This will print:
    ///
    /// ```text
    /// [2022-01-01 12:00:00.000] main.rs:10:
    /// "Hello, world!"
    /// 42
    /// ```
    macro_rules! show {
        ($file:expr, $line:expr, $($args: expr),*) => {
            $(
                print!("[{}] {}:{}:\n{:?}\n", chrono::prelude::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(), $file, $line, $args);
            )*
        }
    }
}

