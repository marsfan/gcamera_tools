/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Top-Level logic for processing an image.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use crate::debug_components::DebugComponents;
use crate::errors::GCameraError;
use crate::jpeg_components::JpegMarker;
use crate::jpeg_components::JpegSegment;
use crate::xmp::XMPData;
use std::convert::TryFrom;
use std::fs;
use std::io::Write;

/// Struct holding all the data for a single image.
#[derive(Debug, PartialEq, Eq)]
pub struct CameraImage {
    /// Vector of the segments in the JPEG portion of the image.
    jpeg_segments: Vec<JpegSegment>,
    /// The camera debug information stored in the image.
    debug_components: DebugComponents,
}

impl CameraImage {
    /// Load a camera image from a file on the disk.
    ///
    /// # Arguments
    /// * `filepath`: The path to the file to load.
    ///
    /// # Returns
    /// Instance of the structure, or an error code.
    pub fn from_file(filepath: String) -> Result<Self, GCameraError> {
        let contents = fs::read(filepath);
        return match contents {
            Ok(c) => Self::try_from(c),
            Err(_) => Err(GCameraError::ImageReadError),
        };
    }

    /// Get the entire JPEG image portion as bytes.
    ///
    /// # Returns
    /// The entire JPEG image portion as a vector of bytes.
    pub fn jpeg_to_bytes(&self) -> Vec<u8> {
        return self
            .jpeg_segments
            .iter()
            .flat_map(|segment| return segment.to_bytes())
            .collect();
    }

    /// Save the JPEG component of the image.
    ///
    /// # Arguments
    /// * `filepath`: Path to save the image to.
    ///
    /// # Returns
    /// Result of saving the file.
    pub fn save_image(&self, filepath: String) -> Result<(), GCameraError> {
        return std::fs::File::create(filepath)
            .map_err(|_| return GCameraError::ImageWriteError)?
            .write_all(&self.jpeg_to_bytes())
            .map_err(|_| return GCameraError::ImageWriteError);
    }

    /// Save the debug data from the image.
    ///
    /// # Arguments
    /// * `filepath`: Path to save the data to
    ///
    /// # Returns
    /// Result from saving the file.
    pub fn save_debug_data(self, filepath: String) -> Result<(), GCameraError> {
        return self
            .debug_components
            .save_data(filepath)
            .map_err(|_| return GCameraError::DebugDataWriteError);
    }

    /// Get the XMP data from the image
    ///
    /// # Returns
    /// The XMP as XML data, or an error message.
    pub fn get_xmp(&self) -> Result<XMPData, String> {
        for segment in self.jpeg_segments.iter() {
            let xmp_string = segment.as_xmp_str();
            if let Some(xmp_string) = xmp_string {
                return XMPData::try_from(xmp_string);
            }
        }

        return Err(String::from("Could not find XMP data"));
    }
}

// Implementation of TryFrom for CameraImage
impl TryFrom<Vec<u8>> for CameraImage {
    type Error = GCameraError;

    /// Create a new instance from a vector of bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes`: The bytes to create the image from.
    ///
    /// # Returns
    /// Result holding the created instance, or an error message
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes[0..2] != vec![0xFF, 0xD8] {
            return Err(GCameraError::Other {
                msg: String::from("Not a valid JPEG file."),
            });
        }

        // FIXME: Figure out how to do this without mutable?
        let mut jpeg_segments: Vec<JpegSegment> = Vec::new();
        jpeg_segments.push(JpegSegment::from_bytes(&bytes, 0)?);
        let mut offset = 0;

        while !matches!(jpeg_segments.last().unwrap().marker, JpegMarker::EOI) {
            let prev = jpeg_segments.last().unwrap();
            offset += prev.byte_count();
            jpeg_segments.push(JpegSegment::from_bytes(&bytes, offset).unwrap());
            // FIXME: Remove unwrap
        }

        for segment in jpeg_segments.iter() {
            let _ = segment.byte_count();
        }

        offset += jpeg_segments.last().unwrap().byte_count();
        let debug_components = DebugComponents::try_from(&bytes[offset..])?;

        return Ok(Self {
            jpeg_segments,
            debug_components,
        });
    }
    // fn from(bytes: Vec<u8>) -> Result<Self, &'static str> {
    //     return Err("ABC");
    // }
}

#[cfg(test)]
mod test {
    use crate::debug_components::DebugChunk;

    use super::*;

    /// Tests for the from_bytes method
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
                jpeg_segments: vec![
                    JpegSegment::from_bytes(&[0xFF, 0xD8], 0).unwrap(),
                    JpegSegment::from_bytes(&[0xFF, 0xD9], 0).unwrap()
                ],
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
                }
            })
        );
    }

    /// Test case where the file magic is incorrect
    #[test]
    fn test_bad_magic() {
        let bytes = vec![0xFF, 0xAA];
        let function_result = CameraImage::try_from(bytes);
        assert_eq!(
            function_result,
            Err(GCameraError::Other {
                msg: String::from("Not a valid JPEG file.")
            })
        );
    }

    /// Test getting the bytes for the JPEG image portion.
    #[test]
    fn test_to_bytes() {
        let image = CameraImage {
            jpeg_segments: vec![
                JpegSegment::from_bytes(&[0xFF, 0xD8], 0).unwrap(),
                JpegSegment::from_bytes(&[0xFF, 0xD9], 0).unwrap(),
            ],
            debug_components: DebugComponents {
                aecdebug: {
                    DebugChunk {
                        magic: String::from("aecDebug"),
                        data: String::from("hi").as_bytes().to_vec(),
                    }
                },
                afdebug: DebugChunk {
                    magic: String::from("afDebug"),
                    data: String::from("bye").as_bytes().to_vec(),
                },
                awbdebug: DebugChunk {
                    magic: String::from("awbDebug"),
                    data: String::from("123").as_bytes().to_vec(),
                },
            },
        };

        assert_eq!(image.jpeg_to_bytes(), vec![0xFF, 0xD8, 0xFF, 0xD9]);
    }
}
