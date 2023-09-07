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
use clap::Parser;

#[derive(Parser, Debug, Eq, PartialEq)]
#[command(author, version, about = "Utility for working with photos take with Google Camera", long_about = None)]
pub struct Arguments {
    /// Path to the image to process
    // FIXME: Why is this showing up after positional arguments?
    #[arg(index = 1)]
    pub input_path: String, // Path to search

    /// Path to save just the JPEG image to
    #[arg(short, long)]
    pub image_output: Option<String>,

    /// Path to save the debug data to
    #[arg(short, long)]
    pub debug_output: Option<String>,

    /// Path to save the motion video to
    #[arg(short, long)]
    pub motion_output: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Document tests
    // TODO: Tests for each optional argument

    #[test]
    fn test_valid_args_no_optionals() {
        let input_args = vec!["/bin/gcamera_tools", "motion_photo.jpg"];
        let parsed_args = Arguments::parse_from(input_args);

        let expected_result = Arguments {
            input_path: String::from("motion_photo.jpg"),
            image_output: None,
            debug_output: None,
            motion_output: None,
        };
        assert_eq!(parsed_args, expected_result);
    }

    // FIXME: Validate the error output somehow
    #[test]
    #[should_panic]
    fn test_not_enough_args() {
        let input_args = vec!["/bin/gcamera_tools"];
        let parsed_args = Arguments::try_parse_from(input_args);
        parsed_args.unwrap();
    }

    // FIXME: Validate the error output somehow
    #[test]
    #[should_panic]
    fn test_too_many_args() {
        let input_args = vec!["/bin/gcamera_tools", "motion_photo.jpg", "second_photo.jpg"];
        let parsed_args = Arguments::try_parse_from(input_args);
        parsed_args.unwrap();
    }
}
