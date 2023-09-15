/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Logic for the entire JPEG image.

use crate::errors::GCameraError;
use crate::jpeg_components::{JpegMarker, JpegSegment};
use crate::xmp::XMPData;

#[derive(PartialEq, Eq, Debug)]
pub struct JpegImage {
    pub segments: Vec<JpegSegment>,
}

impl JpegImage {
    /// Get the size of the image in bytes
    ///
    /// # Returns
    /// The size of the image in bytes
    pub fn image_size(&self) -> usize {
        return self
            .segments
            .iter()
            .map(|segment| return segment.byte_count())
            .sum();
    }

    /// Convert the image to bytes.
    ///
    /// # Returns
    /// The JPEG image as a vector of bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        return self
            .segments
            .iter()
            .flat_map(|segment| return Vec::from(segment))
            .collect();
    }

    /// Get the XMP data from the image
    ///
    /// # Returns
    /// The XMP as `XMPData`.
    ///
    /// # Errors
    /// Will return an error if there is no XMP data in the image
    pub fn get_xmp(&self) -> Result<XMPData, GCameraError> {
        for segment in &self.segments {
            let xmp_string = segment.as_xmp_str();
            if let Some(xmp_string) = xmp_string {
                return XMPData::try_from(xmp_string);
            }
        }

        return Err(GCameraError::NoXMPData);
    }
}

impl TryFrom<&Vec<u8>> for JpegImage {
    type Error = GCameraError;

    /// Create a new instance from a vector of bytes.
    ///
    /// # Arguments:
    /// * `bytes`: The bytes to create the image from
    ///
    /// # Returns
    /// Resulting holding the created image, or an error message.
    fn try_from(bytes: &Vec<u8>) -> Result<Self, Self::Error> {
        if bytes[0..2] != vec![0xFF, 0xD8] {
            return Err(GCameraError::InvalidJpegMagic);
        }

        // FIXME: Figure out how to do this without mutable?
        let mut segments: Vec<JpegSegment> = Vec::new();
        segments.push(JpegSegment::from_bytes(bytes)?);
        let mut offset = 0;

        while !matches!(segments.last().unwrap().marker, JpegMarker::EOI) {
            let prev = segments.last().unwrap();
            offset += prev.byte_count();
            segments.push(JpegSegment::from_bytes(&bytes[offset..])?);
        }

        return Ok(JpegImage { segments });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Test converting the segment to vector of bytes
    #[test]
    fn test_to_bytes() {
        let image = JpegImage {
            segments: vec![
                JpegSegment::from_bytes(&[0xFF, 0xD8]).unwrap(),
                JpegSegment::from_bytes(&[0xFF, 0xD9]).unwrap(),
            ],
        };

        assert_eq!(image.as_bytes(), vec![0xFF, 0xD8, 0xFF, 0xD9])
    }

    /// Test case for when there JPEG magic is invalid
    #[test]
    fn test_invalid_jpeg_magic() {
        let test_bytes = vec![0xFF, 0xDD, 0xAA, 0xBB];
        let image = JpegImage::try_from(&test_bytes);

        assert_eq!(image, Err(GCameraError::InvalidJpegMagic));
    }

    /// Test for when there is no XMP Data segment.
    #[test]
    fn test_no_xmp() {
        let image = JpegImage {
            segments: vec![JpegSegment::from_bytes(&[0xFF, 0xD8]).unwrap()],
        };

        let xmp_data = image.get_xmp();
        assert_eq!(xmp_data, Err(GCameraError::NoXMPData));
    }
}
