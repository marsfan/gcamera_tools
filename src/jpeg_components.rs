//! Logic for working with the actual JPEG image
#![deny(clippy::implicit_return)]
// TODO: Can this be made to only apply to the enum?
#![allow(clippy::upper_case_acronyms)] // Allow this one so we can have capital enum for JPEG marker
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

enum NewJpegSegment {
    TEM { length: u16, data: Vec<u8> },
    SOF0 { length: u16, data: Vec<u8> }, // TODO: Proper Parsing
    SOF1 { length: u16, data: Vec<u8> },
    SOF2 { length: u16, data: Vec<u8> },
    SOF3 { length: u16, data: Vec<u8> },
    DHT { length: u16, data: Vec<u8> }, // TODO: Proper Parsing
    SOF5 { length: u16, data: Vec<u8> },
    SOF6 { length: u16, data: Vec<u8> },
    SOF7 { length: u16, data: Vec<u8> },
    SOI, // No components other than magic and marker
    EOI, // No components other than magic and marker
    SOS { length: u16, data: Vec<u8> },
    DQT { length: u16, data: Vec<u8> }, // TODO: Proper Parsing
    DNL { length: u16, data: Vec<u8> },
    DRI { length: u16, data: Vec<u8> },
    DHP { length: u16, data: Vec<u8> },
    APP0 { length: u16, data: Vec<u8> }, // TODO: Proper Parsing
    APP1 { length: u16, data: Vec<u8> }, // TODO: Proper Parsing
    APP2 { length: u16, data: Vec<u8> },
    APP3 { length: u16, data: Vec<u8> },
    APP4 { length: u16, data: Vec<u8> },
    APP5 { length: u16, data: Vec<u8> },
    APP6 { length: u16, data: Vec<u8> },
    APP7 { length: u16, data: Vec<u8> },
    APP8 { length: u16, data: Vec<u8> },
    APP9 { length: u16, data: Vec<u8> },
    APP10 { length: u16, data: Vec<u8> },
    APP11 { length: u16, data: Vec<u8> },
    APP12 { length: u16, data: Vec<u8> },
    APP13 { length: u16, data: Vec<u8> },
    APP14 { length: u16, data: Vec<u8> },
    APP15 { length: u16, data: Vec<u8> },
    COM { length: u16, data: Vec<u8> },
}

impl NewJpegSegment {
    /// Create a new segment from bytes.
    ///
    /// # Arguments
    /// * `bytes`: The bytes to create the segment from.
    /// * `offset`: Offset into bytes to start creating the segment at.
    ///
    /// # Returns
    /// Result containing either the created segment, or an error message.
    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self, &'static str> {
        let magic = bytes[offset];
        let marker = bytes[offset + 1];
        if magic != 0xFF {
            return Err("Invalid magic byte at start of segment.");
        }

        if marker == 0xD8 {
            return Ok(Self::SOI);
        } else if marker == 0xD9 {
            return Ok(Self::EOI);
        } else {
            let length: u16 = ((bytes[offset + 2] as u16) << 8) | (bytes[offset + 3] as u16);
            let data_length = length - 2;
            let data_start = offset + 4;
            let data_end = offset + (data_length as usize);

            return match marker {
                0x01 => Ok(Self::TEM {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC0 => Ok(Self::SOF0 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC1 => Ok(Self::SOF1 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC2 => Ok(Self::SOF2 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC3 => Ok(Self::SOF3 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC4 => Ok(Self::DHT {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC5 => Ok(Self::SOF5 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC6 => Ok(Self::SOF6 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xC7 => Ok(Self::SOF7 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xDA => Ok(Self::SOS {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xDB => Ok(Self::DQT {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xDC => Ok(Self::DNL {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xDD => Ok(Self::DRI {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xDE => Ok(Self::DHP {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE0 => Ok(Self::APP0 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE1 => Ok(Self::APP1 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE2 => Ok(Self::APP2 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE3 => Ok(Self::APP3 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE4 => Ok(Self::APP4 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE5 => Ok(Self::APP5 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE6 => Ok(Self::APP6 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE7 => Ok(Self::APP7 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE8 => Ok(Self::APP8 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xE9 => Ok(Self::APP9 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xEA => Ok(Self::APP10 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xEB => Ok(Self::APP11 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xEC => Ok(Self::APP12 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xED => Ok(Self::APP13 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xEE => Ok(Self::APP14 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xEF => Ok(Self::APP15 {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                0xFE => Ok(Self::COM {
                    length,
                    data: bytes[data_start..data_end].to_vec(),
                }),
                _ => Err("Unknown JPEG Marker."),
            };
        }
    }

    /// Return the bytes for the marker based on the segment type
    fn to_u8(&self) -> u8 {
        return match self {
            Self::TEM { .. } => 0x01,
            Self::SOF0 { .. } => 0xC0,
            Self::SOF1 { .. } => 0xC1,
            Self::SOF2 { .. } => 0xC2,
            Self::SOF3 { .. } => 0xC3,
            Self::DHT { .. } => 0xC4,
            Self::SOF5 { .. } => 0xC5,
            Self::SOF6 { .. } => 0xC6,
            Self::SOF7 { .. } => 0xC7,
            Self::SOI => 0xD8,
            Self::EOI => 0xD9,
            Self::SOS { .. } => 0xDA,
            Self::DQT { .. } => 0xDB,
            Self::DNL { .. } => 0xDC,
            Self::DRI { .. } => 0xDD,
            Self::DHP { .. } => 0xDE,
            Self::APP0 { .. } => 0xE0,
            Self::APP1 { .. } => 0xE1,
            Self::APP2 { .. } => 0xE2,
            Self::APP3 { .. } => 0xE3,
            Self::APP4 { .. } => 0xE4,
            Self::APP5 { .. } => 0xE5,
            Self::APP6 { .. } => 0xE6,
            Self::APP7 { .. } => 0xE7,
            Self::APP8 { .. } => 0xE8,
            Self::APP9 { .. } => 0xE9,
            Self::APP10 { .. } => 0xEA,
            Self::APP11 { .. } => 0xEB,
            Self::APP12 { .. } => 0xEC,
            Self::APP13 { .. } => 0xED,
            Self::APP14 { .. } => 0xEE,
            Self::APP15 { .. } => 0xEF,
            Self::COM { .. } => 0xFE,
        };
    }

    fn to_bytes(&self) -> Vec<u8> {
        return match self {
            Self::TEM { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOF0 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOF1 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOF2 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOF3 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::DHT { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOF5 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOF6 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOF7 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::SOI => vec![0xFF, self.to_u8()],
            Self::EOI => vec![0xFF, self.to_u8()],
            Self::SOS { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::DQT { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::DNL { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::DRI { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::DHP { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP0 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP1 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP2 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP3 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP4 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP5 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP6 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP7 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP8 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP9 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP10 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP11 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP12 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP13 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP14 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::APP15 { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
            Self::COM { length, data } => [
                &[0xFF],
                &[self.to_u8()],
                length.to_be_bytes().to_vec().as_slice(),
                data.as_slice(),
            ]
            .concat(),
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
/// Offset that the next marker is at, or an error message
fn find_next_segment(bytes: &[u8], start_addr: usize) -> Result<usize, &'static str> {
    let bytes_chunk = bytes[start_addr..].to_vec();
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
            JpegMarker::SOI => 0,
            JpegMarker::EOI => 0,
            JpegMarker::SOS => find_next_segment(bytes, offset + 2)?,
            _ => (bytes[offset + 2] as usize) << 8 | (bytes[offset + 3] as usize),
        };

        return Ok(JpegSegment {
            magic: bytes[offset],
            marker,
            length: length + 2,
            file_offset: offset,
            last_offset: offset + length + 2,
            data: match length {
                0 => vec![],
                _ => bytes[offset + 4..offset + 2 + length].to_vec(),
            },
        });
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
