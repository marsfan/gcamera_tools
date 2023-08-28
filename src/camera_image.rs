use crate::debug_components::DebugComponents;
use crate::jpeg_components::JpegMarker;
use crate::jpeg_components::JpegSegment;
use std::io::Write;

pub struct CameraImage {
    jpeg_segments: Vec<JpegSegment>,
    debug_components: DebugComponents,
}

impl CameraImage {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if bytes[0..2] != vec![0xFF, 0xD8] {
            return Err("Not a valid JPEG File.");
        }

        let mut jpeg_segments: Vec<JpegSegment> = Vec::new();
        jpeg_segments.push(JpegSegment::from_bytes(&bytes, 0));
        while !matches!(jpeg_segments.last().unwrap().marker, JpegMarker::EOI) {
            let prev = jpeg_segments.last().unwrap();
            jpeg_segments.push(JpegSegment::from_bytes(&bytes, prev.last_offset));
        }

        let debug_components =
            DebugComponents::from_bytes(&bytes[jpeg_segments.last().unwrap().last_offset..]);

        return Ok(Self {
            jpeg_segments,
            debug_components,
        });
    }

    pub fn save_image(&self, filepath: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(filepath)?;
        for segment in &self.jpeg_segments {
            file.write_all(&segment.to_bytes())?;
        }
        return Ok(());
    }

    pub fn save_debug_data(self, filepath: &str) -> std::io::Result<()> {
        return self.debug_components.write_data(filepath);
    }
}
