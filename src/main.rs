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

    let image = CameraImage::from_file(&args.input_path).unwrap_or_else(|err| {
        eprintln!("Problem Parsing Image: {err}");
        exit(1);
    });

    // Save the JPEG image if requested
    if args.image_output {
        let output_path = args.input_path.with_extension("image.jpg");
        image
            .save_image(String::from(output_path.to_str().unwrap()))
            .unwrap_or_else(|err| {
                eprintln!("Problem Saving JPEG Image: {err}");
                exit(1)
            })
    }

    // Save the debug data if requested.
    if args.debug_output {
        let output_path = args.input_path.with_extension("debug.bin");
        image
            .save_debug_data(String::from(output_path.to_str().unwrap()))
            .unwrap_or_else(|err| {
                eprintln!("Problem Saving Debug Data: {err}");
                exit(1)
            })
    }
    // Save the motion photo if requested
    if args.motion_output {
        panic!("Motion extracting is not supported yet")
    }

    if args.info {
        image.print_debug_info();
        image.print_resource_info();
    }
}
