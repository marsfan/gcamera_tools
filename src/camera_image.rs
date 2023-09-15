/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Top-Level logic for processing an image.
use crate::debug_components::DebugComponents;
use crate::errors::GCameraError;
use crate::jpeg_image::JpegImage;
use crate::xmp::SemanticType;
use std::convert::TryFrom;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Struct holding all the data for a single image.
#[derive(Debug, PartialEq, Eq)]
pub struct CameraImage {
    /// Vector of the segments in the JPEG portion of the image.
    image: JpegImage,

    /// The camera debug information stored in the image.
    debug_components: DebugComponents,

    /// The total size of the loaded image
    total_size: usize,
}

impl CameraImage {
    /// Load a camera image from a file on the disk.
    ///
    /// # Arguments
    /// * `filepath`: The path to the file to load.
    ///
    /// # Returns
    /// Instance of the structure, or an error code.
    ///
    /// # Errors
    /// Will return an error if reading the image from disk fails.
    pub fn from_file(filepath: &PathBuf) -> Result<Self, GCameraError> {
        return match fs::read(filepath) {
            Ok(contents) => Self::try_from(contents),
            Err(_) => Err(GCameraError::ImageReadError),
        };
    }

    /// Save the JPEG component of the image.
    ///
    /// # Arguments
    /// * `filepath`: Path to save the image to.
    ///
    /// # Returns
    /// Result of saving the file.
    ///
    /// # Errors
    /// Will error if writing the data to disk fails
    pub fn save_image(&self, filepath: PathBuf) -> Result<(), GCameraError> {
        return std::fs::File::create(filepath)
            .map_err(|_| return GCameraError::ImageWriteError)?
            .write_all(&self.image.as_bytes())
            .map_err(|_| return GCameraError::ImageWriteError);
    }

    /// Save the debug data from the image.
    ///
    /// # Arguments
    /// * `filepath`: Path to save the data to
    ///
    /// # Returns
    /// Result from saving the file.
    ///
    /// # Errors
    /// Will error if writing the data to the disk fails.
    pub fn save_debug_data(&self, filepath: PathBuf) -> Result<(), GCameraError> {
        return self
            .debug_components
            .save_data(filepath)
            .map_err(|_| return GCameraError::DebugDataWriteError);
    }

    /// Print out some information about the file.
    /// This is useful for basic debugging.
    pub fn print_debug_info(&self) {
        println!(
            "The main JPEG image has {} segments, for a total of {} bytes.",
            self.image.segments.len(),
            self.image.image_size(),
        );

        println!(
            "The debug section is a total of {} bytes in size.",
            self.debug_components.size()
        );

        if let Ok(xmp) = self.image.get_xmp() {
            println!(
                "There is a total of {} resources in the file according to the XMP data.",
                xmp.resources.len()
            );

            for (index, resource) in xmp.resources.iter().enumerate() {
                println!(
                    "\tResource {index} has a semantic of '{:?}'",
                    resource.semantic
                );
            }
        } else {
            println!("XMP data not found");
        }
    }

    /// Print info about resource offsets.
    ///
    /// # Panics
    /// Will panic if XMP data is not found, resource does not have a length,
    /// or resource does not have padding.
    pub fn print_resource_info(&self) {
        // FIXME: Work out how to integrate this into a new struct, so that it is not just being printed out
        // FIXME: Don't process the primary resource, which is the main JPEG image

        // Computing the start point of each resource.
        // Since we know the length of each resource, we can work backwards
        // through the resources, subtracting the size of each from an
        // variable that starts out at the total size of the image we are
        // parsing.
        let xmp_data = self.image.get_xmp().unwrap();
        let mut length_accumulation = self.total_size;
        for (index, resource) in xmp_data.resources.iter().enumerate().rev() {
            if resource.semantic != SemanticType::Primary {
                length_accumulation -= resource.length.unwrap();
                println!("Resource {index} starts at {length_accumulation}");

                // Also have to account for the padding between each resource
                // that's the point of this
                length_accumulation -= resource.padding.unwrap();
            }
        }
    }
}

// Implementation of TryFrom for CameraImage
impl TryFrom<Vec<u8>> for CameraImage {
    type Error = GCameraError;

    /// Create a new instance from a vector of bytes.
    ///
    /// # Arguments
    /// * `bytes`: The bytes to create the image from.
    ///
    /// # Returns
    /// Result holding the created instance, or an error message
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes[0..2] != vec![0xFF, 0xD8] {
            return Err(GCameraError::InvalidJpegMagic);
        }

        let image = JpegImage::try_from(&bytes)?;

        let debug_components = DebugComponents::try_from(&bytes[image.image_size()..])?;

        return Ok(Self {
            image,
            debug_components,
            total_size: bytes.len(),
        });
    }
}

#[cfg(test)]
mod test {
    use crate::{debug_components::DebugChunk, jpeg_components::JpegSegment};

    use super::*;

    /// Tests for the `from_bytes` method
    #[test]
    fn test_from_bytes() {
        let bytes = vec![
            0xFF, 0xD8, 0xFF, 0xD9, 0x61, 0x65, 0x63, 0x44, 0x65, 0x62, 0x75, 0x67, 0x68, 0x69,
            0x61, 0x66, 0x44, 0x65, 0x62, 0x75, 0x67, 0x62, 0x79, 0x65, 0x61, 0x77, 0x62, 0x44,
            0x65, 0x62, 0x75, 0x67, 0x31, 0x32, 0x33,
        ];
        let image = CameraImage::try_from(bytes);

        assert_eq!(
            image,
            Ok(CameraImage {
                image: JpegImage {
                    segments: vec![
                        JpegSegment::from_bytes(&[0xFF, 0xD8]).unwrap(),
                        JpegSegment::from_bytes(&[0xFF, 0xD9]).unwrap()
                    ]
                },
                debug_components: DebugComponents {
                    aecdebug: {
                        DebugChunk {
                            magic: String::from("aecDebug"),
                            data: String::from("hi").as_bytes().to_vec(),
                        }
                    },
                    afdebug: DebugChunk {
                        magic: String::from("afDebug"),
                        data: String::from("bye").as_bytes().to_vec()
                    },
                    awbdebug: DebugChunk {
                        magic: String::from("awbDebug"),
                        data: String::from("123").as_bytes().to_vec()
                    }
                },
                total_size: 35,
            })
        );
    }

    /// Test case where the file magic is incorrect
    #[test]
    fn test_bad_magic() {
        let bytes = vec![0xFF, 0xAA];
        let function_result = CameraImage::try_from(bytes);
        assert_eq!(function_result, Err(GCameraError::InvalidJpegMagic));
    }
}
