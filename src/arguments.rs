#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use std::env;

pub struct Arguments {
    pub input_path: String,
}

impl Arguments {
    pub fn from_cli() -> Result<Arguments, &'static str> {
        let mut args = env::args();
        // Skip over the executable path
        args.next();

        // Parse the input path argument.
        let input_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Path to image not supplied."),
        };

        // Check for remaining arguments
        if args.next().is_some() {
            return Err("To many arguments supplied.");
        }

        return Ok(Arguments { input_path });
    }
}
