// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
pub use build_print::*;
pub use owo_colors::*;

#[macro_export]
macro_rules! build_print {
    ($topic:expr) => {
        ::std::println!("cargo:warning=\x1b[2K\r{}", $topic.bold());
    };
    ($topic:expr, $($arg:tt)+) => {
        ::std::println!("cargo:warning=\x1b[2K\r   {}: {}", $topic.bold(), ::std::format!($($arg)*));
    }
}

#[macro_export]
macro_rules! echo {
    ($component:expr, cyan, $($arg:tt)* ) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.cyan(), $($arg)*)
        }
    };
    ($component:expr, blue, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.blue(), $($arg)*)
        }
    };
    ($component:expr, orange, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.orange(), $($arg)*)
        }
    };
    ($component:expr, purple, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.purple(), $($arg)*)
        }
    };
    ($component:expr, red, $( $arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.red(), $($arg)*)
        }
    };
    ($component:expr, green, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.green(), $($arg)*)
        }
    };
    ($component:expr, yellow, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.yellow(), $($arg)*)
        }
    };


}
