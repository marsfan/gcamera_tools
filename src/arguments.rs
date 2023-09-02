/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Argument Parser for the command line tool.
//!
//! This module provides the logic used by the command-line tool to read
//! arguments from the command line.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use std::env;

/// Structure holding the parsed arguments.
#[derive(Debug, Eq, PartialEq)]
pub struct Arguments {
    /// Path of the image to process.
    pub input_path: String,
}

impl Arguments {
    /// Parse the arguments from an iterator.
    ///
    /// # Arguments
    /// * `args`: Iterator of arguments to parse.
    ///
    /// # Returns
    /// The parsed arguments, or an error messsage if parsing failed.
    fn parse<T>(mut args: T) -> Result<Self, &'static str>
    where
        T: Iterator<Item = String>,
    {
        // Skip over the executable path
        args.next();

        let input_path: String = match args.next() {
            Some(arg) => arg,
            None => return Err("Path to image not supplied."),
        };

        // Check for remaining arguments
        if args.next().is_some() {
            return Err("To many arguments supplied.");
        }

        return Ok(Self { input_path });
    }
    /// Parse the arguments from the command line.
    ///
    /// # Returns
    /// A result that containing either the parsed arguments, or an error
    /// message.
    pub fn from_cli() -> Result<Self, &'static str> {
        return Self::parse(env::args());
    }
}

#[cfg(test)]
mod tests {
    use super::Arguments;

    #[test]
    fn test_valid_args() {
        let input_args = ["/bin/gcamera_tools", "motion_photo.jpg"]
            .iter()
            .map(|s| s.to_string());
        let parsed_args = Arguments::parse(input_args);

        let expected_result = Ok(Arguments {
            input_path: String::from("motion_photo.jpg"),
        });

        assert_eq!(parsed_args, expected_result);
    }

    #[test]
    fn test_no_enough_args() {
        let input_args = ["/bin/gcamera_tools"].iter().map(|s| s.to_string());
        let parsed_args = Arguments::parse(input_args);

        assert_eq!(parsed_args, Err("Path to image not supplied."));
    }

    #[test]
    fn test_too_many_args() {
        let input_args = ["/bin/gcamera_tools", "motion_photo.jpg", "second_photo.jpg"]
            .iter()
            .map(|s| s.to_string());
        let parsed_args = Arguments::parse(input_args);

        assert_eq!(parsed_args, Err("To many arguments supplied."));
    }
}
