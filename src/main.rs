/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use clap::Parser;
use gcamera_tools::arguments::Arguments;
use gcamera_tools::camera_image::CameraImage;
use std::process::exit;

/// Main function that is run from the command line.
fn main() {
    // Parse command line arguments
    let args = Arguments::parse();

    let image = CameraImage::from_file(args.input_path).unwrap_or_else(|err| {
        eprintln!("Problem Parsing Image: {err}");
        exit(1);
    });

    // Save the JPEG image if the user provides a save path.
    if let Some(output_path) = args.image_output {
        image.save_image(output_path).unwrap()
    }

    // Save the debug data if the user provides a save path.
    if let Some(output_path) = args.debug_output {
        image.save_debug_data(output_path).unwrap()
    }
    // Save the motion photo if the user provides a save path
    if args.motion_output.is_some() {
        panic!("Motion extracting is not supported yet")
    }

    if args.info {
        image.print_debug_info();
    }
}
