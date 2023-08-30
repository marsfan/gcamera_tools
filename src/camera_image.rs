//! Top-Level logic for processing an image.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use crate::debug_components::DebugComponents;
use crate::jpeg_components::JpegMarker;
use crate::jpeg_components::JpegSegment;
use std::io::Write;

/// Struct holding all the data for a single image.
pub struct CameraImage {
    /// Vector of the segments in the JPEG portion of the image.
    jpeg_segments: Vec<JpegSegment>,
    /// The camera debug information stored in the image.
    debug_components: DebugComponents,
}

impl CameraImage {
    /// Create a new instance from a vector of bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes`: The bytes to create the image from.
    ///
    /// # Returns
    /// Result holding the created instance, or an error message
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if bytes[0..2] != vec![0xFF, 0xD8] {
            return Err("Not a valid JPEG File.");
        }

        // FIXME: Figure out how to do this would mutable?
        let mut jpeg_segments: Vec<JpegSegment> = Vec::new();
        jpeg_segments.push(JpegSegment::from_bytes(&bytes, 0)?);
        let mut offset = 0;

        while !matches!(jpeg_segments.last().unwrap().marker, JpegMarker::EOI) {
            let prev = jpeg_segments.last().unwrap();
            offset += prev.byte_count();
            jpeg_segments.push(JpegSegment::from_bytes(&bytes, offset)?);
        }

        for segment in jpeg_segments.iter() {
            let _ = segment.byte_count();
        }

        offset += jpeg_segments.last().unwrap().byte_count();
        let debug_components = DebugComponents::from_bytes(&bytes[offset..])?;

        return Ok(Self {
            jpeg_segments,
            debug_components,
        });
    }

    /// Save the JPEG component of the image.
    ///
    /// # Arguments
    /// * `filepath`: Path to save the image to.
    ///
    /// # Returns
    /// Result of saving the file.
    pub fn save_image(&self, filepath: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(filepath)?;
        for segment in &self.jpeg_segments {
            file.write_all(&segment.to_bytes())?;
        }
        return Ok(());
    }

    /// Save the debug data from the image.
    ///
    /// # Arguments
    /// * `filepath`: Path to save the data to
    ///
    /// # Returns
    /// Result from saving the file.
    pub fn save_debug_data(self, filepath: &str) -> std::io::Result<()> {
        return self.debug_components.save_data(filepath);
    }
}
