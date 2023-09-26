/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Main logic for the command line tool.
#![allow(clippy::print_stderr)]
#![allow(clippy::exit)]

use crate::camera_image::CameraImage;
use crate::cli::arguments::Arguments;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::exit;

/// Create the output file path from the provided argument, or a default value
///
/// If the given argument is `Some()`, use that value as the output path
/// Otherwise, construct the output path by using the name of the input file
/// with a custom extension.
///
/// # Arguments
/// * `argument`: The argument used for a user to manually specify an output path
/// * `input_path`: The input path of the image.
/// * `default_extension`: The extension to use with the input path.
fn create_output_path(
    argument: Option<PathBuf>,
    input_path: &Path,
    default_extension: &str,
) -> PathBuf {
    return match argument {
        Some(path) => path,
        None => input_path.with_extension(default_extension),
    };
}

/// Main function to be called when running the tool.
pub fn tool_main() {
    // Parse command line arguments
    let args = Arguments::parse();

    let image = CameraImage::from_file(&args.input_path).unwrap_or_else(|err| {
        eprintln!("Problem Parsing Image: {err}");
        exit(1);
    });

    // Save the JPEG image if requested
    if args.save_image {
        let output_path = create_output_path(args.image_path, &args.input_path, "image.jpg");
        image.save_image(output_path).unwrap_or_else(|err| {
            eprintln!("Problem Saving JPEG Image: {err}");
            exit(1);
        });
    }

    // Save the debug data if requested.
    if args.save_debug {
        let output_path = create_output_path(args.debug_path, &args.input_path, "debug.bin");
        image.save_debug_data(output_path).unwrap_or_else(|err| {
            eprintln!("Problem Saving Debug Data: {err}");
            exit(1);
        });
    }
    // Save the motion photo if requested
    if args.save_motion {
        let output_path = create_output_path(args.motion_path, &args.input_path, "motion.mp4");
        image.save_motion_video(output_path).unwrap_or_else(|err| {
            eprintln!("Problem Saving Motion Video: {err}");
            exit(1);
        });
    }

    if args.info {
        image.print_debug_info();
    }

    if args.list_resources {
        image.print_resource_list();
    }
}
