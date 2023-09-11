/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Logic for the entire JPEG image.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::errors::GCameraError;
use crate::jpeg_components::{JpegMarker, JpegSegment};

#[derive(PartialEq, Eq, Debug)]
pub struct JpegImage {
    pub segments: Vec<JpegSegment>,
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
            segments.push(JpegSegment::from_bytes(&bytes[offset..]).unwrap());
            // FIXME: Remove unwrap
        }

        return Ok(JpegImage { segments });
    }
}
