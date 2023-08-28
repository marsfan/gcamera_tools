//! Logic for working with the actual JPEG image
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::convert::TryFrom;

/// Enum of the different JPEG segment markers.
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum JpegMarker {
    TEM = 0x01,
    SOF0 = 0xC0,
    SOF1 = 0xC1,
    SOF2 = 0xC2,
    SOF3 = 0xC3,
    DHT = 0xC4,
    SOF5 = 0xC5,
    SOF6 = 0xC6,
    SOF7 = 0xC7,
    SOI = 0xD8,
    EOI = 0xD9,
    SOS = 0xDA,
    DQT = 0xDB,
    DNL = 0xDC,
    DRI = 0xDD,
    DHP = 0xDE,
    APP0 = 0xE0,
    APP1 = 0xE1,
    APP2 = 0xE2,
    APP3 = 0xE3,
    APP4 = 0xE4,
    APP5 = 0xE5,
    APP6 = 0xE6,
    APP7 = 0xE7,
    APP8 = 0xE8,
    APP9 = 0xE9,
    APP10 = 0xEA,
    APP11 = 0xEB,
    APP12 = 0xEC,
    APP13 = 0xED,
    APP14 = 0xEE,
    APP15 = 0xEF,
    COM = 0xFE,
}

impl TryFrom<u8> for JpegMarker {
    type Error = ();
    /// Create an instaced based on the byte value.
    ///
    /// # Arguments
    /// * `value` The byte value to create the instance from.
    ///
    /// # Resturns
    /// Result of creating the instance.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        return match value {
            0x01 => Ok(Self::TEM),
            0xC0 => Ok(Self::SOF0),
            0xC1 => Ok(Self::SOF1),
            0xC2 => Ok(Self::SOF2),
            0xC3 => Ok(Self::SOF3),
            0xC4 => Ok(Self::DHT),
            0xC5 => Ok(Self::SOF5),
            0xC6 => Ok(Self::SOF6),
            0xC7 => Ok(Self::SOF7),
            0xD8 => Ok(Self::SOI),
            0xD9 => Ok(Self::EOI),
            0xDA => Ok(Self::SOS),
            0xDB => Ok(Self::DQT),
            0xDC => Ok(Self::DNL),
            0xDD => Ok(Self::DRI),
            0xDE => Ok(Self::DHP),
            0xE0 => Ok(Self::APP0),
            0xE1 => Ok(Self::APP1),
            0xE2 => Ok(Self::APP2),
            0xE3 => Ok(Self::APP3),
            0xE4 => Ok(Self::APP4),
            0xE5 => Ok(Self::APP5),
            0xE6 => Ok(Self::APP6),
            0xE7 => Ok(Self::APP7),
            0xE8 => Ok(Self::APP8),
            0xE9 => Ok(Self::APP9),
            0xEA => Ok(Self::APP10),
            0xEB => Ok(Self::APP11),
            0xEC => Ok(Self::APP12),
            0xED => Ok(Self::APP13),
            0xEE => Ok(Self::APP14),
            0xEF => Ok(Self::APP15),
            0xFE => Ok(Self::COM),
            _ => Err(()),
        };
    }
}

/// A single JPEG segment.
#[derive(Debug)]
pub struct JpegSegment {
    /// The magic byte for the segment. Should always be 0xFF
    pub magic: u8,

    /// The marker indicating the segment type.
    pub marker: JpegMarker,

    /// The length of the segment
    /// For the SOS segment, this is only the length of the SOS heaader.
    pub length: usize,

    /// Offset into the file that the segment is found at.
    pub file_offset: usize,

    /// Offset in the file to the end of the segment.
    pub last_offset: usize,

    /// The data bytes of the segment.
    pub data: Vec<u8>,
}

/// Linear search for the next JPEG Segment.
///
/// # Arguments
/// * `bytes`: The bytes to search for the next segment.
/// * `start_addr`: Offset into bytes start searching at.
///
/// # Returns
/// Offset that the next marker is at.
fn find_next_segment(bytes: &[u8], start_addr: usize) -> usize {
    let bytes_chunk = bytes[start_addr..].to_vec();
    for (index, byte) in bytes_chunk.iter().enumerate() {
        if byte == &0xFF {
            let marker = JpegMarker::try_from(bytes_chunk[index + 1]);
            if marker.is_ok() {
                return index;
            }
        }
    }
    panic!("Could not find next marker.")
}

impl JpegSegment {
    /// Create a new from bytes.
    ///
    /// # Arguments
    /// * `bytes`: The bytes to create the segment from.
    /// * `offset`: Offset into bytes to start creating the segment at.
    ///
    /// # Returns
    /// Result containing either the created segment, or an error message.
    pub fn from_bytes(bytes: &[u8], offset: usize) -> Self {
        let marker = JpegMarker::try_from(bytes[offset + 1]).unwrap();

        let length = match marker {
            JpegMarker::SOI => 0,
            JpegMarker::EOI => 0,
            JpegMarker::SOS => find_next_segment(bytes, offset + 2),
            _ => (bytes[offset + 2] as usize) << 8 | (bytes[offset + 3] as usize),
        };

        return JpegSegment {
            magic: bytes[offset],
            marker,
            length: length + 2,
            file_offset: offset,
            last_offset: offset + length + 2,
            data: match length {
                0 => vec![],
                _ => bytes[offset + 4..offset + 2 + length].to_vec(),
            },
        };
    }

    /// Convert the segment to bytes.
    ///
    /// # Returns
    /// Bytes of the JPEG segment.
    pub fn to_bytes(&self) -> Vec<u8> {
        let length_bytes: Vec<u8> = match self.marker {
            JpegMarker::SOI => Vec::new(),
            JpegMarker::EOI => Vec::new(),
            JpegMarker::SOS => vec![0x00, 0x0C],
            _ => ((self.length - 2) as u16).to_be_bytes().to_vec(),
        };

        return [
            &[self.magic],
            &[self.marker as u8],
            length_bytes.as_slice(),
            self.data.as_slice(),
        ]
        .concat();
    }
}
