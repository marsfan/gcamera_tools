/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Logic for working with camera debug information in an image.

use std::fs::File;
use std::{io::Write, path::PathBuf};

use crate::errors::GCameraError;

/// A single chunk of debug data.
#[derive(Debug, PartialEq, Eq)]
pub struct DebugChunk {
    /// The magic at the start of the chunk
    pub magic: String,
    /// The data in the chunk
    pub data: Vec<u8>,
}

impl DebugChunk {
    /// Serialize the chunk back into binary bytes.
    ///
    /// # Returns
    /// The chunk as a vector of bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        return [self.magic.as_bytes(), &self.data].concat();
    }

    /// Get the size of the chunk.
    ///
    /// # Returns:
    /// The size of the chunk, in bytes
    pub fn size(&self) -> usize {
        return self.magic.as_bytes().len() + self.data.len();
    }
}

/// Find the start index of the given magic using a linear search.
///
/// # Arguments
/// * `data`: The data to search through for the magic.
/// # `magic`: The magic string to search for.
///
/// # Returns
///
/// Result holding either the index of the start of the magic, or
/// an error string.
fn find_magic_start(data: &[u8], magic: &str) -> Option<usize> {
    // End point must be total length minus magic length, or we we attempt to
    // read outside the array.
    let magic_bytes = magic.as_bytes();

    let search_result = data
        .windows(magic_bytes.len())
        .enumerate()
        .find(|(_, window)| return window == &magic_bytes);

    return search_result.map(|(index, _)| return index);
}

/// All of the debug information from the image.
#[derive(Debug, PartialEq, Eq)]
pub struct DebugComponents {
    /// Contents of the aecDebug portion
    pub aecdebug: Option<DebugChunk>,

    /// Contents of the afDebug portion
    pub afdebug: Option<DebugChunk>,

    /// Contents of the awbDebug portion.
    pub awbdebug: Option<DebugChunk>,
}

impl DebugComponents {
    /// Save the data to a file.
    ///
    /// # Arguments
    /// * `filepath`: Path to the file to save the data to.
    ///
    /// # Returns
    /// Result of saving the data
    ///
    /// # Errors
    /// Will error if writing the debug data to the disk fails
    pub fn save_data(&self, filepath: PathBuf) -> Result<(), GCameraError> {
        return File::create(filepath)
            .map_err(|error| return GCameraError::DebugDataWriteError { kind: error.kind() })?
            .write_all(&self.as_bytes())
            .map_err(|error| return GCameraError::DebugDataWriteError { kind: error.kind() });
    }

    /// Convert the debug data back into bytes.
    ///
    /// # Returns
    /// The data as a vector of bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        let aec_bytes = match &self.aecdebug {
            Some(data) => data.as_bytes(),
            None => Vec::new(),
        };

        let af_bytes = match &self.afdebug {
            Some(data) => data.as_bytes(),
            None => Vec::new(),
        };

        let awb_bytes = match &self.awbdebug {
            Some(data) => data.as_bytes(),
            None => Vec::new(),
        };
        return [
            aec_bytes, af_bytes, awb_bytes, // self.awbdebug.as_bytes(),
        ]
        .concat();
    }

    /// Get the size of all of the debug components.
    ///
    /// # Returns
    /// The total size of all of the debug components.
    pub fn size(&self) -> usize {
        return self.aecdebug.as_ref().map_or(0, |data| return data.size())
            + self.afdebug.as_ref().map_or(0, |data| return data.size())
            + self.awbdebug.as_ref().map_or(0, |data| return data.size());
    }
}

/// Implementation to create debug components from a slice of bytes.
impl From<&[u8]> for DebugComponents {
    /// Create an instance from the bytes.
    ///
    /// # Arguments
    /// * `bytes`: The bytes to create the instance from.
    ///
    /// # Returns
    /// The created DebugComponents struct
    fn from(bytes: &[u8]) -> Self {
        // TODO: use slice.split_array_ref instead of find_magic_start.
        // slice.split_array_ref is still in nightly only
        let aec_start = find_magic_start(bytes, "aecDebug");
        let af_start = find_magic_start(bytes, "afDebug");
        let awb_start = find_magic_start(bytes, "awbDebug");

        let awb_chunk = awb_start.map(|start| {
            return DebugChunk {
                magic: String::from_utf8(bytes[start..start + 8].to_vec()).unwrap(),
                data: bytes[start + 8..].to_vec(),
            };
        });

        // End point of AF is the start of AWB, or if there is no AWB, the end of the binary.
        let af_end = bytes.len() - awb_chunk.as_ref().map_or(0, |chunk| return chunk.size());

        let af_chunk = af_start.map(|start| {
            return DebugChunk {
                magic: String::from_utf8(bytes[start..start + 7].to_vec()).unwrap(),
                data: bytes[start + 7..af_end].to_vec(),
            };
        });

        // Subtract the af size from the AF end if it exists, otherwise, we propagate af_end.
        let aec_end = af_end - af_chunk.as_ref().map_or(0, |chunk| return chunk.size());

        let aec_chunk = aec_start.map(|start| {
            return DebugChunk {
                magic: String::from_utf8(bytes[start..start + 8].to_vec()).unwrap(),
                data: bytes[start + 8..aec_end].to_vec(),
            };
        });

        return DebugComponents {
            aecdebug: aec_chunk,
            afdebug: af_chunk,
            awbdebug: awb_chunk,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod chunk_tests {
        use super::*;

        /// Test the `as_bytes` method
        #[test]
        fn test_to_bytes() {
            let chunk = DebugChunk {
                magic: String::from("hello"),
                data: vec![0x01, 0x02, 0x03, 0xFF, 0xAB],
            };

            let expected = vec![0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x01, 0x02, 0x03, 0xFF, 0xAB];

            assert_eq!(chunk.as_bytes(), expected);
        }

        /// test the size() method on a debug chunk
        #[test]
        fn test_size() {
            let chunk = DebugChunk {
                magic: String::from("hello"),
                data: vec![0x01, 0x02, 0x03, 0xFF, 0xAB],
            };
            assert_eq!(chunk.size(), 10);
        }
    }

    mod find_magic_start_tests {
        use super::*;

        /// Test finding magic.
        #[test]
        fn test_magic_found() {
            let test_bytes = [0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x68, 0x69, 0x03, 0xFF, 0xAB];
            let magic = "hi";

            let found_offset = find_magic_start(&test_bytes, magic);

            assert_eq!(found_offset, Some(5));
        }

        /// Test not being able to find the magic
        #[test]
        fn test_magic_not_found() {
            let test_bytes = [0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x01, 0x02, 0x03, 0xFF, 0xAB];
            let magic = "hi";

            let function_result = find_magic_start(&test_bytes, magic);

            assert_eq!(function_result, None);
        }
    }

    mod test_debug_components {
        use super::*;

        /// Test not being able to find magic bytes.
        #[test]
        fn test_no_magic_found() {
            let test_bytes = "hello how are you".as_bytes();
            let result = DebugComponents::try_from(test_bytes);

            assert_eq!(
                result,
                Ok(DebugComponents {
                    aecdebug: None,
                    afdebug: None,
                    awbdebug: None
                })
            );
        }

        /// Test successfully creation
        #[test]
        fn test_successful_creation() {
            let test_bytes = "aecDebug abc afDebug def awbDebug ghi".as_bytes();
            let result = DebugComponents::try_from(test_bytes);

            let expected_struct = DebugComponents {
                aecdebug: Some(DebugChunk {
                    magic: String::from("aecDebug"),
                    data: vec![0x20, 0x61, 0x62, 0x63, 0x20],
                }),
                afdebug: Some(DebugChunk {
                    magic: String::from("afDebug"),
                    data: vec![0x20, 0x64, 0x65, 0x66, 0x20],
                }),
                awbdebug: Some(DebugChunk {
                    magic: String::from("awbDebug"),
                    data: vec![0x20, 0x67, 0x68, 0x69],
                }),
            };

            assert_eq!(result, Ok(expected_struct));
        }

        /// Test converting to bytes.
        #[test]
        fn test_to_bytes() {
            let debug_components = DebugComponents {
                aecdebug: Some(DebugChunk {
                    magic: String::from("aecDebug"),
                    data: vec![0x20, 0x61, 0x62, 0x63, 0x20],
                }),
                afdebug: Some(DebugChunk {
                    magic: String::from("afDebug"),
                    data: vec![0x20, 0x64, 0x65, 0x66, 0x20],
                }),
                awbdebug: Some(DebugChunk {
                    magic: String::from("awbDebug"),
                    data: vec![0x20, 0x67, 0x68, 0x69],
                }),
            };

            let generated_bytes = debug_components.as_bytes();

            let expected_bytes = "aecDebug abc afDebug def awbDebug ghi".as_bytes();

            assert_eq!(generated_bytes, expected_bytes);
        }

        /// Test getting the overall size of all of the debug components
        #[test]
        fn test_size() {
            let debug_components = DebugComponents {
                aecdebug: Some(DebugChunk {
                    magic: String::from("aecDebug"),
                    data: vec![0x20, 0x61, 0x62, 0x63, 0x20],
                }),
                afdebug: Some(DebugChunk {
                    magic: String::from("afDebug"),
                    data: vec![0x20, 0x64, 0x65, 0x66, 0x20],
                }),
                awbdebug: Some(DebugChunk {
                    magic: String::from("awbDebug"),
                    data: vec![0x20, 0x67, 0x68, 0x69],
                }),
            };

            assert_eq!(debug_components.size(), 37);
        }
    }
}
