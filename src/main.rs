/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
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
    if args.save_image {
        let output_path = match args.image_path {
            Some(image_path) => image_path,
            None => args.input_path.with_extension("image.jpg"),
        };
        image.save_image(output_path).unwrap_or_else(|err| {
            eprintln!("Problem Saving JPEG Image: {err}");
            exit(1)
        })
    }

    // Save the debug data if requested.
    if args.save_debug {
        let output_path = match args.debug_path {
            Some(debug_path) => debug_path,
            None => args.input_path.with_extension("debug.bin"),
        };
        image.save_debug_data(output_path).unwrap_or_else(|err| {
            eprintln!("Problem Saving Debug Data: {err}");
            exit(1)
        })
    }
    // Save the motion photo if requested
    if args.save_motion {
        panic!("Motion extracting is not supported yet")
    }

    if args.info {
        image.print_debug_info();
    }
}
