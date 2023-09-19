/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Argument Parser for the command line tool.
//!
//! This module provides the logic used by the command-line tool to read
//! arguments from the command line.
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Eq, PartialEq)]
#[command(author, version, about = "Utility for working with photos take with Google Camera", long_about = None)]
pub struct Arguments {
    /// Path to the image to process
    #[arg(index = 1)]
    pub input_path: PathBuf, // Path to search

    /// Save the primary image to a new file.
    #[arg(short = 'i', long)]
    pub save_image: bool,

    /// Optional path to save the primary image to
    #[arg(long, requires = "save_image")]
    pub image_path: Option<PathBuf>,

    /// Save the debug data in a new file
    #[arg(short = 'd', long)]
    pub save_debug: bool,

    /// Optional path to save debug data to
    #[arg(long, requires = "save_debug")]
    pub debug_path: Option<PathBuf>,

    /// Flag to save the motion photo video
    #[arg(short = 'm', long)]
    pub save_motion: bool,

    /// Optional path to save the motion video to
    #[arg(long, requires = "save_motion")]
    pub motion_path: Option<PathBuf>,

    /// Print out some information about the file
    #[arg(short = 'I', long)]
    pub info: bool,
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
            input_path: PathBuf::from("motion_photo.jpg"),
            save_image: false,
            image_path: None,
            save_debug: false,
            debug_path: None,
            save_motion: false,
            motion_path: None,
            info: false,
        };
        assert_eq!(parsed_args, expected_result);
    }

    /// Test with `image_output` set.
    #[test]
    fn test_image_output() {
        let input_args = vec!["/bin/gcamera_tools", "motion_photo.jpg", "--save-image"];
        let parsed_args = Arguments::parse_from(input_args);
        let expected_results = Arguments {
            input_path: PathBuf::from("motion_photo.jpg"),
            save_image: true,
            image_path: None,
            save_debug: false,
            debug_path: None,
            save_motion: false,
            motion_path: None,
            info: false,
        };
        assert_eq!(parsed_args, expected_results);
    }

    /// Test with `save_debug` set
    #[test]
    fn test_save_debug() {
        let input_args = vec!["/bin/gcamera_tools", "motion_photo.jpg", "--save-debug"];
        let parsed_args = Arguments::parse_from(input_args);
        let expected_results = Arguments {
            input_path: PathBuf::from("motion_photo.jpg"),
            save_image: false,
            image_path: None,
            save_debug: true,
            debug_path: None,
            save_motion: false,
            motion_path: None,
            info: false,
        };
        assert_eq!(parsed_args, expected_results);
    }

    /// Test with video-output set
    #[test]
    fn test_video_output() {
        let input_args = vec!["/bin/gcamera_tools", "motion_photo.jpg", "--save-motion"];
        let parsed_args = Arguments::parse_from(input_args);
        let expected_results = Arguments {
            input_path: PathBuf::from("motion_photo.jpg"),
            save_image: false,
            image_path: None,
            save_debug: false,
            debug_path: None,
            save_motion: true,
            motion_path: None,
            info: false,
        };
        assert_eq!(parsed_args, expected_results);
    }
    /// Test with info set
    #[test]
    fn test_info_enabled() {
        let input_args = vec!["/bin/gcamera_tools", "motion_photo.jpg", "-I"];
        let parsed_args = Arguments::parse_from(input_args);
        let expected_results = Arguments {
            input_path: PathBuf::from("motion_photo.jpg"),
            save_image: false,
            image_path: None,
            save_debug: false,
            debug_path: None,
            save_motion: false,
            motion_path: None,
            info: true,
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

    /// Use clap's built in unit test ability.
    #[test]
    fn verify_arguments() {
        use clap::CommandFactory;
        Arguments::command().debug_assert();
    }
}
