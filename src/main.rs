#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use gcamera_tools::arguments::Arguments;
use gcamera_tools::debug_components::DebugComponents;
use gcamera_tools::jpeg_components::{JpegMarker, JpegSegment};
use std::fs;
use std::io::Write;
use std::process::exit;

fn main() {
    let jpeg_magic = vec![0xFF, 0xD8];

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

    if contents[0..2] != jpeg_magic {
        eprintln!("Provided file is not a JPEG image.");
    }

    // Get the JPEG segments from the image.
    let mut jpeg_segments: Vec<JpegSegment> = Vec::new();

    jpeg_segments.push(JpegSegment::from_bytes(&contents, 0));
    while !matches!(jpeg_segments.last().unwrap().marker, JpegMarker::EOI) {
        let prev = jpeg_segments.last().unwrap();
        jpeg_segments.push(JpegSegment::from_bytes(&contents, prev.last_offset))
    }

    // Get the debug segments from the image.
    let debug_components =
        DebugComponents::from_bytes(&contents[jpeg_segments.last().unwrap().last_offset..]);

    // Save the segments.
    let mut file = std::fs::File::create("just_photo.jpg").unwrap();
    for segment in jpeg_segments {
        file.write_all(&segment.to_bytes()).unwrap();
    }

    let mut file = std::fs::File::create("just_debug.bin").unwrap();
    file.write_all(&debug_components.to_bytes()).unwrap();
}
