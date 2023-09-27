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

    /// Print out a list of the additional resources
    #[arg(short = 'l', long)]
    pub list_resources: bool,
}

impl Arguments {
    /// Create the output file path from the provided argument, or a default value
    ///
    /// If the given argument is `Some()`, use that value as the output path
    /// Otherwise, construct the output path by using the name of the input file
    /// with a custom extension.
    ///
    /// # Arguments
    /// * `argument`: The argument used for a user to manually specify an output path
    /// * `extension`: The extension to use with the input path.
    pub fn create_output_path(&self, argument: &Option<PathBuf>, extension: &str) -> PathBuf {
        return match argument {
            Some(path) => path.clone(),
            None => self.input_path.with_extension(extension),
        };
    }
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::*;

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

    /// Test that the `image_path` arg without `save_image` fails.
    #[test]
    fn test_image_path_missing_flag() {
        let input_args = vec![
            "/bin/gcamera_tools",
            "motion_photo.jpg",
            "--image-path",
            "image.jpg",
        ];
        let parsed_args = Arguments::try_parse_from(input_args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err().kind(),
            ErrorKind::MissingRequiredArgument
        );
    }
    /// Test that the `debug_path` arg without `save_debug` fails.
    #[test]
    fn test_debug_path_missing_flag() {
        let input_args = vec![
            "/bin/gcamera_tools",
            "motion_photo.jpg",
            "--debug-path",
            "image.bin",
        ];
        let parsed_args = Arguments::try_parse_from(input_args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err().kind(),
            ErrorKind::MissingRequiredArgument
        );
    }
    /// Test that the `motion_path` arg without `save_motion` fails.
    #[test]
    fn test_motion_path_missing_flag() {
        let input_args = vec![
            "/bin/gcamera_tools",
            "motion_photo.jpg",
            "--motion-path",
            "image.mp4",
        ];
        let parsed_args = Arguments::try_parse_from(input_args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err().kind(),
            ErrorKind::MissingRequiredArgument
        );
    }

    /// Test `create_output_path` when the default should be used
    #[test]
    fn test_create_output_path_default() {
        let parsed_args = Arguments::parse_from(vec!["/bin/gcamera_tools", "motion_photo.jpg"]);
        let output_path = parsed_args.create_output_path(&None, "motion.mp4");
        assert_eq!(output_path, PathBuf::from("motion_photo.motion.mp4"));
    }

    /// Test `create_output_path` when an argument is provided.
    #[test]
    fn test_create_output_path_no_default() {
        let parsed_args = Arguments::parse_from(vec!["/bin/gcamera_tools", "motion_photo.jpg"]);
        let output_path =
            parsed_args.create_output_path(&Some(PathBuf::from("hello.mp4")), "motion.mp4");
        assert_eq!(output_path, PathBuf::from("hello.mp4"));
    }

    /// Use clap's built in unit test ability.
    #[test]
    fn verify_arguments() {
        use clap::CommandFactory;
        Arguments::command().debug_assert();
    }
}
