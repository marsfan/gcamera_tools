//! Argument Parser for the command line tool.
//!
//! This module provides the logic used by the command-line tool to read
//! arguments from the command line.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use std::env;

/// Structure holding the parsed arguments.
pub struct Arguments {
    /// Path of the image to process.
    pub input_path: String,
}

impl Arguments {
    /// Parse the arguments from the command line.
    ///
    /// # Returns:
    ///     A result that containing either the parsed arguments, or an error
    ///     message.
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
