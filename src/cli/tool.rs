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
use std::process::exit;

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
        let output_path = match args.image_path {
            Some(image_path) => image_path,
            None => args.input_path.with_extension("image.jpg"),
        };
        image.save_image(output_path).unwrap_or_else(|err| {
            eprintln!("Problem Saving JPEG Image: {err}");
            exit(1);
        });
    }

    // Save the debug data if requested.
    if args.save_debug {
        let output_path = match args.debug_path {
            Some(debug_path) => debug_path,
            None => args.input_path.with_extension("debug.bin"),
        };
        image.save_debug_data(output_path).unwrap_or_else(|err| {
            eprintln!("Problem Saving Debug Data: {err}");
            exit(1);
        });
    }
    // Save the motion photo if requested
    if args.save_motion {
        let output_path = match args.motion_path {
            Some(motion_path) => motion_path,
            None => args.input_path.with_extension("motion.mp4"),
        };
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
