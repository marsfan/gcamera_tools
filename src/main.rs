#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use gcamera_tools::debug_components::DebugComponents;
use gcamera_tools::jpeg_components::{JpegMarker, JpegSegment};
use std::env;
use std::fs;
use std::io::Write;
use std::process::exit;
struct Arguments {
    pub input_path: String,
}

impl Arguments {
    pub fn from_cli() -> Result<Arguments, &'static str> {
        let mut args = env::args();
        // Skip over the executable path
        args.next();

        // Parse the input path argument.
        let input_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Path to image not supplied."),
        };

        // Check for remaining arguments
        if args.next().is_some() {
            return Err("To many arguments supplied.");
        }

        return Ok(Arguments { input_path });
    }
}

fn main() {
    let jpeg_magic = vec![0xFF, 0xD8];

    // Get the path to the image
    let args = Arguments::from_cli().unwrap_or_else(|err| {
        eprintln!("{err}");
        exit(1);
    });

    let contents = fs::read(args.input_path).unwrap_or_else(|err| {
        eprintln!("Problem reading image: {err}");
        exit(1);
    });

    if contents[0..2] != jpeg_magic {
        eprintln!("Provided file is not a JPEG image.");
    }

    let mut jpeg_segments: Vec<JpegSegment> = Vec::new();

    jpeg_segments.push(JpegSegment::from_bytes(&contents, 0));
    while !matches!(jpeg_segments.last().unwrap().marker, JpegMarker::EOI) {
        let prev = jpeg_segments.last().unwrap();
        jpeg_segments.push(JpegSegment::from_bytes(&contents, prev.last_offset))
    }

    let debug_components =
        DebugComponents::from_bytes(&contents[jpeg_segments.last().unwrap().last_offset..]);

    for (index, segment) in jpeg_segments.iter().enumerate() {
        println!("Segment {index} has marker {:?}", segment.marker);
    }
    println!(
        "{}, {}, {}",
        debug_components.aecdebug.magic,
        debug_components.afdebug.magic,
        debug_components.awbdebug.magic
    );

    let mut file = std::fs::File::create("just_photo.jpg").unwrap();
    for segment in jpeg_segments.iter() {
        file.write_all(&[segment.magic]).unwrap();
        file.write_all(&[segment.marker as u8]).unwrap();
        match segment.marker {
            JpegMarker::SOI => {}
            JpegMarker::EOI => {}
            JpegMarker::SOS => file.write_all(&[0x00, 0x0C]).unwrap(),
            _ => file
                .write_all(&((segment.length - 2) as u16).to_be_bytes())
                .unwrap(),
        };
        file.write_all(&segment.data).unwrap();
    }
}
