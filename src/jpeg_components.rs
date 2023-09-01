//! Logic for working with the actual JPEG image
#![deny(clippy::implicit_return)]
// TODO: Can this be made to only apply to the enum?
#![allow(clippy::needless_return)]

/// Enum of the different JPEG segment markers.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Eq, PartialEq)]
pub struct JpegSegment {
    /// The magic byte for the segment. Should always be 0xFF
    magic: u8,

    /// The marker indicating the segment type.
    pub marker: JpegMarker,

    /// The length of the segment
    /// For the SOS segment, this is only the length of the SOS header.
    /// Since SOI and EOI don't have data bytes, this is an Option
    /// # Note
    /// The length includes its own length, so the number of bytes in the
    /// `data` variable is two less than the value in the length
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

        let data = data_length.map(|len| return bytes[offset + 4..offset + 2 + len].to_vec());

        if (data.is_none() && length.is_none()) || (data.is_some() && length.is_some()) {
            return Ok(JpegSegment {
                magic: bytes[offset],
                marker,
                length,
                data,
            });
        } else {
            panic!("Data and length must either both be None, or both be some. This should not be possible.");
        }
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

    /// Get XMP data
    ///
    /// If this segment is the XMP data segment, this will return
    /// A string containing the XMP data. Otherwise it returns None
    ///
    /// # Returns
    /// The XMP Data as a string, or None
    pub fn as_xmp_str(&self) -> Option<String> {
        let xmp_marker = "http://ns.adobe.com/xap/1.0/".as_bytes();
        if matches!(self.marker, JpegMarker::APP1)
            && self.data.as_ref().unwrap().starts_with(xmp_marker)
        {
            return Some(
                String::from_utf8(self.data.as_ref().unwrap()[xmp_marker.len() + 1..].to_vec())
                    .unwrap(),
            );
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod jpeg_marker_tests {
        use super::*;
        /// Test converting each valid enum option both to and from
        /// the given u8 value.
        #[test]
        fn test_to_from_u8_success() {
            let test_cases = vec![
                (0x01, JpegMarker::TEM),
                (0xC0, JpegMarker::SOF0),
                (0xC1, JpegMarker::SOF1),
                (0xC2, JpegMarker::SOF2),
                (0xC3, JpegMarker::SOF3),
                (0xC4, JpegMarker::DHT),
                (0xC5, JpegMarker::SOF5),
                (0xC6, JpegMarker::SOF6),
                (0xC7, JpegMarker::SOF7),
                (0xD8, JpegMarker::SOI),
                (0xD9, JpegMarker::EOI),
                (0xDA, JpegMarker::SOS),
                (0xDB, JpegMarker::DQT),
                (0xDC, JpegMarker::DNL),
                (0xDD, JpegMarker::DRI),
                (0xDE, JpegMarker::DHP),
                (0xE0, JpegMarker::APP0),
                (0xE1, JpegMarker::APP1),
                (0xE2, JpegMarker::APP2),
                (0xE3, JpegMarker::APP3),
                (0xE4, JpegMarker::APP4),
                (0xE5, JpegMarker::APP5),
                (0xE6, JpegMarker::APP6),
                (0xE7, JpegMarker::APP7),
                (0xE8, JpegMarker::APP8),
                (0xE9, JpegMarker::APP9),
                (0xEA, JpegMarker::APP10),
                (0xEB, JpegMarker::APP11),
                (0xEC, JpegMarker::APP12),
                (0xED, JpegMarker::APP13),
                (0xEE, JpegMarker::APP14),
                (0xEF, JpegMarker::APP15),
                (0xFE, JpegMarker::COM),
            ];
            for (byte, marker) in test_cases {
                assert_eq!(JpegMarker::from_u8(byte), Ok(marker));
                assert_eq!(marker as u8, byte);
            }
        }

        /// Test getting an error for invalid byte input
        #[test]
        fn test_invalid_from_u8() {
            assert_eq!(JpegMarker::from_u8(0xFF), Err("Unknown JPEG segment type."));
        }
    }

    mod find_next_segment_tests {
        use crate::jpeg_components::find_next_segment;

        /// Test valid discovery of next segment.
        #[test]
        fn test_valid_next_segment() {
            let test_bytes = [0x01, 0x02, 0x03, 0x04, 0x04, 0x06, 0xFF, 0xD9, 0xAB, 0xCD];
            let found_index = find_next_segment(&test_bytes);
            assert_eq!(found_index, Ok(6));
        }

        /// Test no valid segment bytes at all
        #[test]
        fn test_no_found_segment() {
            let test_bytes = [0x01, 0x02, 0x03, 0x04, 0x04, 0x06, 0xAB, 0xCD];
            assert_eq!(
                find_next_segment(&test_bytes),
                Err("Could not find next marker.")
            );
        }
        /// Test where magic is valid, but marker is not
        #[test]
        fn test_no_found_segment_valid_magic() {
            let test_bytes = [0x01, 0x02, 0x03, 0x04, 0x04, 0x06, 0xFF, 0xFF, 0xAB, 0xCD];
            assert_eq!(
                find_next_segment(&test_bytes),
                Err("Could not find next marker.")
            );
        }
    }

    mod test_jpeg_segment {
        use super::*;

        mod test_from_bytes {
            use super::*;

            /// Test creation of a SOI segment
            /// This allows a test case where both length and data are None
            #[test]
            fn test_soi() {
                let bytes = [0xFF, 0xD8];
                let result = JpegSegment::from_bytes(&bytes, 0);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        magic: 0xFF,
                        marker: JpegMarker::SOI,
                        length: None,
                        data: None
                    })
                )
            }

            /// Test creation of a EOI segment
            /// This allows a test case where both length and data are None
            #[test]
            fn test_eoi() {
                let bytes = [0xFF, 0xD9];
                let result = JpegSegment::from_bytes(&bytes, 0);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        magic: 0xFF,
                        marker: JpegMarker::EOI,
                        length: None,
                        data: None
                    })
                )
            }

            /// Test creation of a normal segment
            /// i.e one with both length and data bits.
            #[test]
            fn test_create_general() {
                let bytes = [0xFF, 0xFE, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04];
                let result = JpegSegment::from_bytes(&bytes, 0);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        magic: 0xFF,
                        marker: JpegMarker::COM,
                        length: Some(4),
                        data: Some(vec![0x01, 0x02])
                    })
                );
            }

            /// Test creation of a SOS segment where there is a following segment.
            #[test]
            fn test_create_sos_with_more() {
                let bytes = [
                    0xFF, 0xDA, 0x00, 0x04, 0x01, 0x02, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0xFF, 0xDA,
                ];

                let result = JpegSegment::from_bytes(&bytes, 0);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        magic: 0xFF,
                        marker: JpegMarker::SOS,
                        length: Some(4),
                        data: Some(vec![
                            0x01, 0x02, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
                        ])
                    })
                )
            }

            /// Test creation of a SOS segment where there is not a following segment.

            /// Test creation with an offset.
            /// Using a SOI since that's a really simple test case
            #[test]
            fn test_create_offset() {
                let bytes = [0x01, 0x02, 0x03, 0xFF, 0xD8];
                let result = JpegSegment::from_bytes(&bytes, 3);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        magic: 0xFF,
                        marker: JpegMarker::SOI,
                        length: None,
                        data: None
                    })
                )
            }
        }

        /// Tests for teh byte_count function.
        mod test_byte_count {
            use super::*;

            ///Test when length and data are None
            #[test]
            fn test_no_data() {
                let segment = JpegSegment {
                    magic: 0xFF,
                    marker: JpegMarker::APP0,
                    length: None,
                    data: None,
                };

                assert_eq!(segment.byte_count(), 2);
            }

            /// Test when length and data hold values.
            #[test]
            fn test_with_data() {
                let segment = JpegSegment {
                    magic: 0xFF,
                    marker: JpegMarker::APP0,
                    length: Some(0x04),
                    data: Some(vec![0x01, 0x02]),
                };

                assert_eq!(segment.byte_count(), 6);
            }
        }

        mod test_to_bytes {
            use super::*;

            /// Test case for a segment that is an EOI type, which has
            /// neither length or data components.
            #[test]
            fn test_eoi() {
                let segment = JpegSegment {
                    magic: 0xFF,
                    marker: JpegMarker::EOI,
                    length: None,
                    data: None,
                };

                assert_eq!(segment.to_bytes(), vec![0xFF, 0xD9]);
            }

            /// Test case for a segment that contains both the length and
            /// data sections.
            #[test]
            fn test_normal() {
                let segment = JpegSegment {
                    magic: 0xFF,
                    marker: JpegMarker::APP0,
                    length: Some(0x04),
                    data: Some(vec![0x01, 0x02]),
                };

                assert_eq!(segment.to_bytes(), vec![0xFF, 0xE0, 0x00, 0x04, 0x01, 0x02]);
            }
        }
    }
}
