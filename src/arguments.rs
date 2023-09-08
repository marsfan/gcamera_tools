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

    /// Test with none of the optional arguments
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

    /// Test with image_output set.
    #[test]
    fn test_image_output() {
        let input_args = vec![
            "/bin/gcamera_tools",
            "motion_photo.jpg",
            "--image-output",
            "just_photo.jpg",
        ];
        let parsed_args = Arguments::parse_from(input_args);
        let expected_results = Arguments {
            input_path: String::from("motion_photo.jpg"),
            image_output: Some(String::from("just_photo.jpg")),
            debug_output: None,
            motion_output: None,
        };
        assert_eq!(parsed_args, expected_results);
    }

    /// Test with debug_output set
    #[test]
    fn test_debug_output() {
        let input_args = vec![
            "/bin/gcamera_tools",
            "motion_photo.jpg",
            "--debug-output",
            "debug_data.bin",
        ];
        let parsed_args = Arguments::parse_from(input_args);
        let expected_results = Arguments {
            input_path: String::from("motion_photo.jpg"),
            image_output: None,
            debug_output: Some(String::from("debug_data.bin")),
            motion_output: None,
        };
        assert_eq!(parsed_args, expected_results);
    }

    /// Test with video-output set
    #[test]
    fn test_video_output() {
        let input_args = vec![
            "/bin/gcamera_tools",
            "motion_photo.jpg",
            "--motion-output",
            "motion.mp4",
        ];
        let parsed_args = Arguments::parse_from(input_args);
        let expected_results = Arguments {
            input_path: String::from("motion_photo.jpg"),
            image_output: None,
            debug_output: None,
            motion_output: Some(String::from("motion.mp4")),
        };
        assert_eq!(parsed_args, expected_results);
    }

    /// Test for when not enough arguments are supplied
    // FIXME: Validate the error output somehow
    #[test]
    #[should_panic]
    fn test_not_enough_args() {
        let input_args = vec!["/bin/gcamera_tools"];
        let parsed_args = Arguments::try_parse_from(input_args);
        parsed_args.unwrap();
    }

    /// Test for when too many positional arguments are supplied
    // FIXME: Validate the error output somehow
    #[test]
    #[should_panic]
    fn test_too_many_args() {
        let input_args = vec!["/bin/gcamera_tools", "motion_photo.jpg", "second_photo.jpg"];
        let parsed_args = Arguments::try_parse_from(input_args);
        parsed_args.unwrap();
    }
}
