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
use crate::errors::GCameraError;
use clap::Parser;

/// Main function to be called when running the tool.
///
/// # Errors
/// Will return an error if the tool fails for any reason.
pub fn tool_main() -> Result<(), GCameraError> {
    // Parse command line arguments
    let args = Arguments::parse();

    let image = CameraImage::from_file(&args.input_path)?;

    // Save the JPEG image if requested
    if args.save_image {
        let output_path = args.create_output_path(&args.image_path, "image.jpg");
        image.save_image(output_path)?;
    }

    // Save the debug data if requested.
    if args.save_debug {
        let output_path = args.create_output_path(&args.debug_path, "debug.bin");
        image.save_debug_data(output_path)?;
    }
    // Save the motion photo if requested
    if args.save_motion {
        let output_path = args.create_output_path(&args.motion_path, "motion.mp4");
        image.save_motion_video(output_path)?;
    }

    if args.info {
        image.print_debug_info();
    }

    if args.list_resources {
        image.print_resource_list();
    }

    return Ok(());
}
