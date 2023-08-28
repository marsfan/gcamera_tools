//! Logic for working with camera debug information in an image.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::io::Write;

/// A single chunk of debug data.
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

// FIXME: Need a way to handle being out of range
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
    for (position, _) in data.iter().enumerate() {
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
    let length = bytes.len();
    let range_end = length - magic.len();
    for (offset, _) in bytes[..range_end].iter().enumerate() {
        if &bytes[offset..offset + magic.len()] == magic {
            return offset;
        }
    }
    return bytes.len() - 1;
}

/// All of the debug information from the image.
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
    pub fn save_data(self, filepath: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(filepath)?;
        file.write_all(&self.to_bytes())?;
        return Ok(());
    }
}
