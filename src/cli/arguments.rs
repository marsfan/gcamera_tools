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

    /// Use clap's built in unit test ability.
    #[test]
    fn verify_arguments() {
        use clap::CommandFactory;
        Arguments::command().debug_assert();
    }
}
