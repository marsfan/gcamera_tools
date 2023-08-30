//! Logic for working with the actual JPEG image
#![deny(clippy::implicit_return)]
// TODO: Can this be made to only apply to the enum?
#![allow(clippy::needless_return)]

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

impl JpegMarker {
    /// Create an instance based on the byte value.
    ///
    /// # Arguments
    /// * `value` The byte value to create the instance from.
    ///
    /// # Resturns
    /// Result of creating the instance, or an error message
    fn from_u8(value: u8) -> Result<Self, &'static str> {
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
            _ => Err("Unknown JPEG segment type."),
        };
    }
}

/// Linear search for the next JPEG Segment.
///
/// # Arguments
/// * `bytes`: The bytes to search for the next segment.
///
/// # Returns
/// Offset that the next marker is at, or an error message
fn find_next_segment(bytes: &[u8]) -> Result<usize, &'static str> {
    let bytes_chunk = bytes.to_vec();
    for (index, byte) in bytes_chunk.iter().enumerate() {
        if byte == &0xFF {
            let marker = JpegMarker::from_u8(bytes_chunk[index + 1]);
            if marker.is_ok() {
                return Ok(index);
            }
        }
    }
    return Err("Could not find next marker.");
}

/// A single JPEG segment.
#[derive(Debug)]
pub struct JpegSegment {
    /// The magic byte for the segment. Should always be 0xFF
    magic: u8,

    /// The marker indicating the segment type.
    pub marker: JpegMarker,

    /// The length of the segment
    /// For the SOS segment, this is only the length of the SOS heaaer.
    /// Since SOI and EOI don't have data bytes, this is an Option
    length: Option<usize>,

    /// The data bytes of the segment.
    /// Since SOI and EOI don't have data bytes, this is an Option
    pub data: Option<Vec<u8>>,
}

impl JpegSegment {
    /// Create a new segment from bytes.
    ///
    /// # Arguments
    /// * `bytes`: The bytes to create the segment from.
    /// * `offset`: Offset into bytes to start creating the segment at.
    ///
    /// # Returns
    /// Result containing either the created segment, or an error message.
    pub fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self, &'static str> {
        let marker = JpegMarker::from_u8(bytes[offset + 1])?;

        let length = match marker {
            JpegMarker::SOI => None,
            JpegMarker::EOI => None,
            _ => Some((bytes[offset + 2] as usize) << 8 | (bytes[offset + 3] as usize)),
        };

        let data_length = match marker {
            JpegMarker::SOI => None,
            JpegMarker::EOI => None,
            JpegMarker::SOS => Some(find_next_segment(&bytes[offset + 2..])?),
            _ => Some((bytes[offset + 2] as usize) << 8 | (bytes[offset + 3] as usize)),
        };

        let data_bytes = match data_length {
            Some(len) => Some(bytes[offset + 4..offset + 2 + len].to_vec()),
            None => None,
        };

        return Ok(JpegSegment {
            magic: bytes[offset],
            marker,
            length,
            data: data_bytes,
        });
    }

    /// Convert the segment to bytes.
    ///
    /// # Returns
    /// Bytes of the JPEG segment.
    pub fn to_bytes(&self) -> Vec<u8> {
        let length_bytes = match self.length {
            Some(length) => (length as u16).to_be_bytes().to_vec(),
            None => Vec::new(),
        };

        let data_bytes = match &self.data {
            Some(data) => data.as_slice(),
            None => &[],
        };

        return [
            &[self.magic],
            &[self.marker as u8],
            length_bytes.as_slice(),
            data_bytes,
        ]
        .concat();
    }

    // TODO: IDK If it is safe to do this. Should probably ask online.
    // pub fn iter(&self) -> impl ExactSizeIterator<Item = u8> {
    //     return self.to_bytes().into_iter();
    // }

    /// Get the total number of bytes in the segment, if it was serialized to bytes
    ///
    /// # Returns
    /// The total number of bytes in the segment, if it was serialized to bytes
    pub fn byte_count(&self) -> usize {
        // Len size is a u16
        let len_size = match self.length {
            Some(_) => 2,
            None => 0,
        };

        let data_size = match &self.data {
            Some(data) => data.len(),
            None => 0,
        };

        // The 2 at the start is for the marker and magic bytes
        return 2 + len_size + data_size;
    }
}
