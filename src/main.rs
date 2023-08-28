#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use gcamera_tools::arguments::Arguments;
use gcamera_tools::camera_image::CameraImage;
use std::fs;
use std::process::exit;

fn main() {
    // Get the path to the image
    let args = Arguments::from_cli().unwrap_or_else(|err| {
        eprintln!("{err}");
        exit(1);
    });

    // Read the file and verify it is a JPEG
    let contents = fs::read(args.input_path).unwrap_or_else(|err| {
        eprintln!("Problem reading image: {err}");
        exit(1);
    });

    // Get the JPEG segments from the image.
    let image = CameraImage::from_bytes(contents).unwrap();

    // Save the separate parts of the image.
    image.save_image("just_photo.jpg").unwrap();
    image.save_debug_data("just_debug.bin").unwrap();
}
