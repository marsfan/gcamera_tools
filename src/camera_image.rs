/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Top-Level logic for processing an image.
use crate::debug_components::DebugComponents;
use crate::errors::GCameraError;
use crate::jpeg::jpeg_image::JpegImage;
use crate::jpeg::xmp::{Item, SemanticType, XMPData};
use std::convert::TryFrom;
use std::fmt::Write as _; // import without risk of name clashing
use std::fs;
use std::io::Write;
use std::path::PathBuf;
/// Struct for a single non-primary resource in the image.
#[derive(Debug, PartialEq, Eq)]
pub struct Resource {
    /// The bytes of the resource.
    pub data: Vec<u8>,

    /// Information about the resource.
    pub info: Item,
}

/// Struct holding all the data for a single image.
#[derive(Debug, PartialEq, Eq)]
pub struct CameraImage {
    /// Vector of the segments in the JPEG portion of the image.
    image: JpegImage,

    /// The camera debug information stored in the image.
    debug_components: DebugComponents,

    /// Extra resources found in the image
    resources: Vec<Resource>,

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
            Err(error) => Err(GCameraError::ImageReadError { kind: error.kind() }),
        };
    }

    /// Get the first resource of the given semantic type
    ///
    /// # Arguments
    /// * `resource_type`: The semantic type of the resource to get.
    ///
    /// # Returns
    /// The first resource that has the matching semantic type
    ///
    /// # Errors
    /// Will error if there are no resources of the given semantic type
    fn get_resource_by_type(&self, resource_type: SemanticType) -> Result<&Resource, GCameraError> {
        for resource in &self.resources {
            if resource.info.semantic == resource_type {
                return Ok(resource);
            }
        }
        return Err(GCameraError::NoResourcesOfType {
            semantic_type: resource_type,
        });
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
            .map_err(|error| return GCameraError::ImageWriteError { kind: error.kind() })?
            .write_all(&self.image.as_bytes())
            .map_err(|error| return GCameraError::ImageWriteError { kind: error.kind() });
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
        return self.debug_components.save_data(filepath);
    }

    /// Save the motion photo from the image.
    ///
    /// # Arguments
    /// * `filepath`: Path to save the video to
    ///
    /// # Returns
    /// Result from saving the file
    ///
    /// # Errors
    /// Will error if writing the video to the disk fails
    pub fn save_motion_video(&self, filepath: PathBuf) -> Result<(), GCameraError> {
        return std::fs::File::create(filepath)
            .map_err(|error| return GCameraError::MotionVideoWriteError { kind: error.kind() })?
            .write_all(&self.get_resource_by_type(SemanticType::MotionPhoto)?.data)
            .map_err(|error| return GCameraError::MotionVideoWriteError { kind: error.kind() });
    }

    /// Get a string of the debug info
    ///
    /// # Returns
    /// A string with the debug info to print
    fn get_debug_info(&self) -> String {
        return format!(
            "\
Number of JPEG segments: {}
JPEG image size:         {}
Debug section size:      {}
Number of resources:     {}",
            self.image.segments.len(),
            self.image.image_size(),
            self.debug_components.size(),
            self.resources.len(),
        );
    }

    /// Print out some information about the file.
    /// This is useful for basic debugging.
    pub fn print_debug_info(&self) {
        println!("{}", self.get_debug_info());
    }

    /// Get a string list of the additional resources.
    ///
    /// # Returns
    /// A string containing a list of the additional resources in the file.
    fn get_resource_str(&self) -> String {
        let mut resource_str = String::new();
        resource_str.push_str("Additional Resources:\n");
        for (index, resource) in self.resources.iter().enumerate() {
            writeln!(
                resource_str,
                "\tResource {index} has a size of {} and is of type '{:?}'",
                resource.data.len(),
                resource.info.semantic
            )
            .unwrap();
        }
        return resource_str;
    }

    /// Print out a list of the additional resources
    pub fn print_resource_list(&self) {
        print!("{}", self.get_resource_str());
    }
}

/// Create resource vector based on XMP Data, and bytes
///
/// # Arguments
/// * `xmp`: The `XMPData` to parse to find the resources.
/// * `bytes`: The bytes to extract resources from
///
/// # Returns
/// Tuple where the first element is a vector of all non-primary resources.
/// and the second element is the offset where the resources start.
fn get_resources_from_xmp(xmp: XMPData, bytes: &[u8]) -> (Vec<Resource>, usize) {
    // TODO: Reduce mutable stuff. Likely using either the `scan` or `fold` methods.
    let mut resources: Vec<Resource> = Vec::new();
    // Accumulator that starts at file end. We will iterate over
    // resources from XMP backwards and use each resource's length and
    // padding members to compute the start of the resource.
    let mut length_accumulator = bytes.len();
    for (_, resource) in xmp.resources.iter().enumerate().rev() {
        // data chunk ends at the previous accumulator values.
        let data_end = length_accumulator;
        length_accumulator -= resource.length.unwrap();
        if resource.semantic != SemanticType::Primary {
            resources.push(Resource {
                data: Vec::from(&bytes[length_accumulator..data_end]),
                info: resource.clone(),
            });

            // Account for any data padding.
            length_accumulator -= resource.padding;
        }
    }
    // Get resources back into correct order when returning
    return (resources.into_iter().rev().collect(), length_accumulator);
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

        let (resources, resources_start) = match image.get_xmp() {
            Ok(xmp_data) => get_resources_from_xmp(xmp_data, &bytes),
            Err(_) => (Vec::new(), bytes.len()),
        };

        let debug_components =
            DebugComponents::try_from(&bytes[image.image_size()..resources_start])?;

        return Ok(Self {
            image,
            debug_components,
            resources,
            total_size: bytes.len(),
        });
    }
}

#[cfg(test)]
mod test {
    use crate::{
        debug_components::DebugChunk,
        jpeg::{jpeg_components::JpegSegment, xmp::MimeType},
    };

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
                resources: Vec::new(),
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

    /// Test the `get_debug_info` function
    #[test]
    fn test_get_debug_info() {
        let test_image = CameraImage {
            image: JpegImage {
                segments: vec![
                    JpegSegment::from_bytes(&[0xFF, 0xD8]).unwrap(),
                    JpegSegment::from_bytes(&[0xFF, 0xD9]).unwrap(),
                ],
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
                    data: String::from("bye").as_bytes().to_vec(),
                },
                awbdebug: DebugChunk {
                    magic: String::from("awbDebug"),
                    data: String::from("123").as_bytes().to_vec(),
                },
            },
            resources: Vec::new(),
            total_size: 35,
        };
        let debug_info = test_image.get_debug_info();
        assert_eq!(
            debug_info,
            String::from(
                "\
Number of JPEG segments: 2
JPEG image size:         4
Debug section size:      31
Number of resources:     0"
            )
        );
    }

    /// Test the `get_resource_str` method
    #[test]
    fn test_get_resource_str() {
        let test_image = CameraImage {
            image: JpegImage {
                segments: vec![
                    JpegSegment::from_bytes(&[0xFF, 0xD8]).unwrap(),
                    JpegSegment::from_bytes(&[0xFF, 0xD9]).unwrap(),
                ],
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
                    data: String::from("bye").as_bytes().to_vec(),
                },
                awbdebug: DebugChunk {
                    magic: String::from("awbDebug"),
                    data: String::from("123").as_bytes().to_vec(),
                },
            },
            resources: vec![
                Resource {
                    data: vec![0x01, 0x02],
                    info: Item {
                        mimetype: MimeType::Mp4,
                        length: Some(2),
                        padding: 0,
                        semantic: SemanticType::MotionPhoto,
                        label: None,

                        uri: None,
                    },
                },
                Resource {
                    data: vec![0x03, 0x04],
                    info: Item {
                        mimetype: MimeType::Jpeg,
                        length: Some(2),
                        padding: 0,
                        semantic: SemanticType::GainMap,
                        label: None,
                        uri: None,
                    },
                },
            ],
            total_size: 39,
        };
        let resource_str = test_image.get_resource_str();
        assert_eq!(
            resource_str,
            String::from(
                "\
Additional Resources:
\tResource 0 has a size of 2 and is of type 'MotionPhoto'
\tResource 1 has a size of 2 and is of type 'GainMap'\n"
            )
        );
    }
}
