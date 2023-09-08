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
use std::fs;
use std::process::exit;

/// Main function that is run from the command line.
fn main() {
    // Parse command line arguments
    let args = Arguments::parse();

    // Read the file and verify it is a JPEG
    let contents = fs::read(args.input_path).unwrap_or_else(|err| {
        eprintln!("Problem reading image: {err}");
        exit(1);
    });

    // Get the JPEG segments from the image.
    let image = CameraImage::from_bytes(contents).unwrap_or_else(|err| {
        eprintln!("Problem parsing image: {err}");
        exit(1);
    });

    // Save the JPEG image if the user provides a save path.
    match args.image_output {
        Some(output_path) => image.save_image(output_path).unwrap(),
        None => {}
    }

    // Save the debug data if the user provides a save path.
    match args.debug_output {
        Some(output_path) => image.save_debug_data(output_path).unwrap(),
        None => {}
    }

    // Save the motion photo if the user provides a save path
    match args.motion_output {
        Some(_) => panic!("Motion extracting is not supported yet"),
        None => {}
    }
}
