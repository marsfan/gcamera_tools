/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Logic for working with camera debug information in an image.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::io::Write;

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
    pub fn to_bytes(self) -> Vec<u8> {
        return [self.magic.into_bytes(), self.data].concat();
    }
}

/// Find the start index of the given magic using a linear search.
///
/// # Arguments
/// * `data`: The data to search through for the magic.
/// # `magic`: The magic byes to search for.
///
/// # Returns
///
/// Result holding either the index of the start of the magic, or
/// an error string.
fn find_magic_start(data: &[u8], magic: &[u8]) -> Result<usize, &'static str> {
    // End point must be total length minus magic length, or we we attempt to
    // read outside the array.
    let loop_end_point = data.len() - magic.len();
    for (position, _) in data[..loop_end_point].iter().enumerate() {
        let last_byte = position + magic.len();
        let chunk = data[position..last_byte].to_vec();
        if chunk == magic {
            return Ok(position);
        }
    }
    return Err("Could not find start of magic.");
}

// TODO: Better logic since there could be other data than just MP4
// TODO: Could possibly use "bytes.window" instead?
/// Search for the end of the awbDebug chunk.
///
/// This searches for either the header for a mp4 section (i.e. for a Motion Photo)
///
/// # Arguments
/// * `bytes`: The bytes to search through.
///
/// # Returns
/// The index of the end of the awbDebug chunk
fn find_awb_end(bytes: &[u8]) -> usize {
    let magic = "\x00\x00\x00\x1cftypisom".as_bytes();

    // Special case for when the total number of bytes is less than
    // the size of the MP4 magic. This means that there is no MP4 magic.
    if bytes.len() < magic.len() {
        return bytes.len();
    }

    // Loop through looking for MP4 magic
    let length = bytes.len();
    let range_end = length - magic.len() + 1;
    for (offset, _) in bytes[..range_end].iter().enumerate() {
        if &bytes[offset..offset + magic.len()] == magic {
            return offset;
        }
    }

    // MP4 magic not found. Size is the total length
    return bytes.len();
}

/// All of the debug information from the image.
#[derive(Debug, PartialEq, Eq)]
pub struct DebugComponents {
    /// Contents of the aecDebug portion
    pub aecdebug: DebugChunk,

    /// Contents of the afDebug portion
    pub afdebug: DebugChunk,

    /// Contents of the awbDebug portion.
    pub awbdebug: DebugChunk,
}

impl DebugComponents {
    /// Create an instance from the bytes.
    ///
    /// # Arguments
    /// * `bytes`: The bytes to create the instance from.
    ///
    /// # Returns
    /// Result containing either the instance, or an error message
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        // TODO: Proper Error Handling
        let aec_start = find_magic_start(bytes, b"aecDebug")?;
        let af_start = find_magic_start(&bytes[aec_start..], b"afDebug")? + aec_start;
        let awb_start = find_magic_start(&bytes[af_start..], b"awbDebug")? + af_start;
        let awb_end = find_awb_end(&bytes[awb_start..]) + awb_start;

        return Ok(DebugComponents {
            aecdebug: DebugChunk {
                magic: String::from_utf8(bytes[aec_start..aec_start + 8].to_vec()).unwrap(),
                data: bytes[aec_start + 8..af_start].to_vec(),
            },
            afdebug: DebugChunk {
                magic: String::from_utf8(bytes[af_start..af_start + 7].to_vec()).unwrap(),
                data: bytes[af_start + 7..awb_start].to_vec(),
            },
            awbdebug: DebugChunk {
                magic: String::from_utf8(bytes[awb_start..awb_start + 8].to_vec()).unwrap(),
                data: bytes[awb_start + 8..awb_end].to_vec(),
            },
        });
    }

    /// Convert the debug data back into bytes.
    ///
    /// # Returns
    /// The data as a vector of bytes.
    pub fn to_bytes(self) -> Vec<u8> {
        return [
            self.aecdebug.to_bytes(),
            self.afdebug.to_bytes(),
            self.awbdebug.to_bytes(),
        ]
        .concat();
    }

    /// Save the data to a file.
    ///
    /// # Arguments
    /// * `filepath`: Path to the file to save the data to.
    ///
    /// # Returns
    /// Result of saving the data
    pub fn save_data(self, filepath: String) -> std::io::Result<()> {
        std::fs::File::create(filepath)?.write_all(&self.to_bytes())?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod chunk_tests {
        use super::*;

        /// Test the to_bytes method
        #[test]
        fn test_to_bytes() {
            let chunk = DebugChunk {
                magic: String::from("hello"),
                data: vec![0x01, 0x02, 0x03, 0xFF, 0xAB],
            };

            let expected = vec![0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x01, 0x02, 0x03, 0xFF, 0xAB];

            assert_eq!(chunk.to_bytes(), expected);
        }
    }

    mod find_magic_start_tests {
        use super::*;

        /// Test finding magic.
        #[test]
        fn test_magic_found() {
            let test_bytes = [0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x01, 0x02, 0x03, 0xFF, 0xAB];
            let magic = [0x01, 0x02];

            let found_offset = find_magic_start(&test_bytes, &magic);

            assert_eq!(found_offset, Ok(5));
        }

        /// Test not being able to find the magic
        #[test]
        fn test_magic_not_found() {
            let test_bytes = [0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x01, 0x02, 0x03, 0xFF, 0xAB];
            let magic = [0x01, 0x03];

            let function_result = find_magic_start(&test_bytes, &magic);

            assert_eq!(function_result, Err("Could not find start of magic."))
        }
    }

    mod find_awb_end_tests {
        use crate::debug_components::find_awb_end;

        /// Test where there is MP4 magic and additional bytes
        #[test]
        fn test_end_from_magic() {
            let test_bytes =
                "hello how are you\x00\x00\x00\x1cftypisom this is more data.".as_bytes();

            let function_result = find_awb_end(&test_bytes);

            assert_eq!(function_result, 17);
        }

        /// Test where there is a MP4 magic, but nothing afterwards
        #[test]
        fn test_end_from_magic_no_trailing() {
            let test_bytes = "hello how are you\x00\x00\x00\x1cftypisom".as_bytes();

            let function_result = find_awb_end(&test_bytes);

            assert_eq!(function_result, 17);
        }

        /// Test where the end of the section is the end of all of the bytes.
        #[test]
        fn test_end_from_vec_end() {
            let test_bytes = "hello how are you.".as_bytes();

            let function_result = find_awb_end(&test_bytes);

            assert_eq!(function_result, 18);
        }

        /// Test case for if the total number of bytes is less than the MP4 magic
        #[test]
        fn test_shorter_than_mp4_magic() {
            let test_bytes = [
                0x61, 0x77, 0x62, 0x44, 0x65, 0x62, 0x75, 0x67, 0x31, 0x32, 0x33,
            ];
            let function_result = find_awb_end(&test_bytes);
            assert_eq!(function_result, 11);
        }
    }

    mod test_debug_components {
        use crate::debug_components::{DebugChunk, DebugComponents};

        /// Test not being able to find magic bytes.
        #[test]
        fn test_no_magic_found() {
            let test_bytes = "hello how are you".as_bytes();
            let result = DebugComponents::from_bytes(test_bytes);

            assert_eq!(result, Err("Could not find start of magic."));
        }

        /// Test successfully creation
        #[test]
        fn test_successful_creation() {
            let test_bytes = "aecDebug abc afDebug def awbDebug ghi".as_bytes();
            let result = DebugComponents::from_bytes(test_bytes);

            let expected_struct = DebugComponents {
                aecdebug: DebugChunk {
                    magic: String::from("aecDebug"),
                    data: vec![0x20, 0x61, 0x62, 0x63, 0x20],
                },
                afdebug: DebugChunk {
                    magic: String::from("afDebug"),
                    data: vec![0x20, 0x64, 0x65, 0x66, 0x20],
                },
                awbdebug: DebugChunk {
                    magic: String::from("awbDebug"),
                    data: vec![0x20, 0x67, 0x68, 0x69],
                },
            };

            assert_eq!(result, Ok(expected_struct));
        }

        /// Test successfully creating when there is MP4 trailing bytes
        #[test]
        fn test_successful_creation_with_mp4() {
            let test_bytes =
                "aecDebug abc afDebug def awbDebug ghi\x00\x00\x00\x1cftypisom".as_bytes();
            let result = DebugComponents::from_bytes(test_bytes);

            let expected_struct = DebugComponents {
                aecdebug: DebugChunk {
                    magic: String::from("aecDebug"),
                    data: vec![0x20, 0x61, 0x62, 0x63, 0x20],
                },
                afdebug: DebugChunk {
                    magic: String::from("afDebug"),
                    data: vec![0x20, 0x64, 0x65, 0x66, 0x20],
                },
                awbdebug: DebugChunk {
                    magic: String::from("awbDebug"),
                    data: vec![0x20, 0x67, 0x68, 0x69],
                },
            };

            assert_eq!(result, Ok(expected_struct));
        }

        /// Test converting to bytes.
        #[test]
        fn test_to_bytes() {
            let debug_components = DebugComponents {
                aecdebug: DebugChunk {
                    magic: String::from("aecDebug"),
                    data: vec![0x20, 0x61, 0x62, 0x63, 0x20],
                },
                afdebug: DebugChunk {
                    magic: String::from("afDebug"),
                    data: vec![0x20, 0x64, 0x65, 0x66, 0x20],
                },
                awbdebug: DebugChunk {
                    magic: String::from("awbDebug"),
                    data: vec![0x20, 0x67, 0x68, 0x69],
                },
            };

            let generated_bytes = debug_components.to_bytes();

            let expected_bytes = "aecDebug abc afDebug def awbDebug ghi".as_bytes();

            assert_eq!(generated_bytes, expected_bytes);
        }
    }
}
