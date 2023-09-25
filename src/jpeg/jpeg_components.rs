/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Logic for working with the actual JPEG image

use crate::errors::GCameraError;

/// Enum of the different JPEG segment markers.
#[allow(clippy::upper_case_acronyms)] // Allowing because names are upper for JPEG segments
#[allow(clippy::missing_docs_in_private_items)] // Allowing since documenting this would be a pain
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum JpegMarker {
    TEM,
    SOF0,
    SOF1,
    SOF2,
    SOF3,
    DHT,
    SOF5,
    SOF6,
    SOF7,
    SOI,
    EOI,
    SOS,
    DQT,
    DNL,
    DRI,
    DHP,
    APP0,
    APP1,
    APP2,
    APP3,
    APP4,
    APP5,
    APP6,
    APP7,
    APP8,
    APP9,
    APP10,
    APP11,
    APP12,
    APP13,
    APP14,
    APP15,
    COM,
}

/// Conversion of a `JpegMarker` into a u8
impl From<JpegMarker> for u8 {
    /// Convert `JpegMarker` to a u8
    ///
    /// # Arguments
    /// * `value`: The value to convert to an integer
    ///
    /// # Returns
    /// Integer form of the marker.
    fn from(value: JpegMarker) -> Self {
        return match value {
            JpegMarker::TEM => 0x01,
            JpegMarker::SOF0 => 0xC0,
            JpegMarker::SOF1 => 0xC1,
            JpegMarker::SOF2 => 0xC2,
            JpegMarker::SOF3 => 0xC3,
            JpegMarker::DHT => 0xC4,
            JpegMarker::SOF5 => 0xC5,
            JpegMarker::SOF6 => 0xC6,
            JpegMarker::SOF7 => 0xC7,
            JpegMarker::SOI => 0xD8,
            JpegMarker::EOI => 0xD9,
            JpegMarker::SOS => 0xDA,
            JpegMarker::DQT => 0xDB,
            JpegMarker::DNL => 0xDC,
            JpegMarker::DRI => 0xDD,
            JpegMarker::DHP => 0xDE,
            JpegMarker::APP0 => 0xE0,
            JpegMarker::APP1 => 0xE1,
            JpegMarker::APP2 => 0xE2,
            JpegMarker::APP3 => 0xE3,
            JpegMarker::APP4 => 0xE4,
            JpegMarker::APP5 => 0xE5,
            JpegMarker::APP6 => 0xE6,
            JpegMarker::APP7 => 0xE7,
            JpegMarker::APP8 => 0xE8,
            JpegMarker::APP9 => 0xE9,
            JpegMarker::APP10 => 0xEA,
            JpegMarker::APP11 => 0xEB,
            JpegMarker::APP12 => 0xEC,
            JpegMarker::APP13 => 0xED,
            JpegMarker::APP14 => 0xEE,
            JpegMarker::APP15 => 0xEF,
            JpegMarker::COM => 0xFE,
        };
    }
}

/// Conversion of a u8 into a `JpegMarker`
impl TryFrom<u8> for JpegMarker {
    type Error = GCameraError;

    /// Create an instance based on the byte value.
    ///
    /// # Arguments
    /// * `value` The byte value to create the instance from.
    ///
    /// # Resturns
    /// Result of creating the instance, or an error message
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
            _ => Err(GCameraError::UnknownJpegMarker { marker_byte: value }),
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
fn find_next_segment(bytes: &[u8]) -> Result<usize, GCameraError> {
    for (index, byte) in bytes.iter().enumerate() {
        if byte == &0xFF {
            let marker = JpegMarker::try_from(bytes[index + 1]);
            if marker.is_ok() {
                return Ok(index);
            }
        }
    }
    return Err(GCameraError::JpegMarkerNotFound);
}

/// A single JPEG segment.
#[derive(Debug, Eq, PartialEq)]
pub struct JpegSegment {
    /// The marker indicating the segment type.
    pub marker: JpegMarker,

    /// The length of the segment
    /// For the SOS segment, this is only the length of the SOS header.
    /// Since SOI and EOI don't have data bytes, this is an Option
    /// # Note
    /// The length includes its own length, so the number of bytes in the
    /// `data` variable is two less than the value in the length
    length: Option<u16>,

    /// The data bytes of the segment.
    /// Since SOI and EOI don't have data bytes, this is an Option
    pub data: Option<Vec<u8>>,
}

impl JpegSegment {
    /// Create a new segment.
    ///
    /// Not to be used for creating the SOS, SOI, or EOI segments.
    ///
    /// # Arguments
    /// * `marker`: Segment marker type.
    /// * `data`: Segment data (as bytes)
    ///
    /// # Returns
    /// Created segment
    pub fn new(marker: JpegMarker, data: Vec<u8>) -> Self {
        #[allow(clippy::wildcard_enum_match_arm)]
        match marker {
            JpegMarker::SOI => {
                return Self {
                    marker,
                    length: None,
                    data: None,
                }
            }
            JpegMarker::EOI => {
                return Self {
                    marker,
                    length: None,
                    data: None,
                }
            }
            JpegMarker::SOS => {
                return Self {
                    marker,
                    length: Some(0x0C),
                    data: Some(data),
                }
            }
            _ => {
                return Self {
                    marker,
                    length: Some((data.len() + 2).try_into().unwrap()),
                    data: Some(data),
                }
            }
        }
    }

    // TODO: Instead use TryFrom?
    /// Create a new segment from bytes.
    ///
    /// # Arguments
    /// * `bytes`: The bytes to create the segment from.
    ///
    /// # Returns
    /// Result containing either the created segment, or an error message.
    ///
    /// # Errors
    /// Will error if creating a `JpegMarker` is not found.
    /// Additionally, if the segment is a SOS segment, will error
    /// if another segment cannot be found after the SOS Segment
    ///
    /// # Panics
    /// Will panic if a segment has one (but not both of), a length
    /// portion, and a data portion, as this is not possible under
    /// the JPEG spec.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, GCameraError> {
        let marker = JpegMarker::try_from(bytes[1])?;

        #[allow(clippy::wildcard_enum_match_arm)]
        let length = match marker {
            JpegMarker::SOI => None,
            JpegMarker::EOI => None,
            _ => Some((u16::from(bytes[2]) << 8) | u16::from(bytes[3])),
        };

        // Allow wildcard matching here since we only care about if we have SOS
        #[allow(clippy::wildcard_enum_match_arm)]
        let data_length = match marker {
            JpegMarker::SOS => Some(find_next_segment(&bytes[2..])?),
            _ => length.map(|val| return usize::from(val)),
        };

        let data = data_length.map(|len| return bytes[4..(2 + len)].to_vec());

        if (data.is_none() && length.is_none()) || (data.is_some() && length.is_some()) {
            return Ok(JpegSegment {
                marker,
                length,
                data,
            });
        } else {
            return Err(GCameraError::LengthDataNotSameOption);
        }
    }

    /// Get the total number of bytes in the segment, if it was serialized to bytes
    ///
    /// # Returns
    /// The total number of bytes in the segment, if it were to be serialized to bytes
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
    ///
    /// # Panics
    /// Will panic if an attempt to parse the data to a UTF8 string
    /// fails.
    pub fn as_xmp_str(&self) -> Option<String> {
        // FIXME: COnstant to share with XMP Module
        let xmp_marker = "http://ns.adobe.com/xap/1.0/".as_bytes();

        // Extract the data from the struct only if the marker is the right type.
        let data = match (self.marker, &self.data) {
            (JpegMarker::APP1, Some(data_bytes)) => data_bytes.clone(),
            (_, _) => Vec::new(),
        };

        // Check for the XMP Marker
        if data.starts_with(xmp_marker) {
            // Parse to string and return
            let xml_offset = xmp_marker.len() + 1;
            let xml_portion = Vec::from(&data[xml_offset..]);
            return Some(String::from_utf8(xml_portion).unwrap());
        } else {
            return None;
        }
    }

    /// Get the segment as a vector of bytes.
    ///
    /// # Returns
    /// The segment as a vector of bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        let length_bytes = match self.length {
            Some(length) => length.to_be_bytes().to_vec(), // FIXME: Get rid of the to_vec call
            None => Vec::new(),
        };

        let data_bytes = match &self.data {
            Some(data) => data.as_slice(),
            None => &[],
        };

        return [
            &[0xFF],
            &[u8::from(self.marker)],
            length_bytes.as_slice(),
            data_bytes,
        ]
        .concat();
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
                assert_eq!(JpegMarker::try_from(byte), Ok(marker));
                assert_eq!(u8::from(marker), byte);
            }
        }

        /// Test getting an error for invalid byte input
        #[test]
        fn test_invalid_from_u8() {
            assert_eq!(
                JpegMarker::try_from(0xFF),
                Err(GCameraError::UnknownJpegMarker { marker_byte: 0xFF })
            );
        }
    }

    mod find_next_segment_tests {
        use super::*;

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
                Err(GCameraError::JpegMarkerNotFound)
            );
        }
        /// Test where magic is valid, but marker is not
        #[test]
        fn test_no_found_segment_valid_magic() {
            let test_bytes = [0x01, 0x02, 0x03, 0x04, 0x04, 0x06, 0xFF, 0xFF, 0xAB, 0xCD];
            assert_eq!(
                find_next_segment(&test_bytes),
                Err(GCameraError::JpegMarkerNotFound)
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
                let result = JpegSegment::from_bytes(&bytes);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        marker: JpegMarker::SOI,
                        length: None,
                        data: None
                    })
                );
            }

            /// Test creation of a EOI segment
            /// This allows a test case where both length and data are None
            #[test]
            fn test_eoi() {
                let bytes = [0xFF, 0xD9];
                let result = JpegSegment::from_bytes(&bytes);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        marker: JpegMarker::EOI,
                        length: None,
                        data: None
                    })
                );
            }

            /// Test creation of a normal segment
            /// i.e one with both length and data bits.
            #[test]
            fn test_create_general() {
                let bytes = [0xFF, 0xFE, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04];
                let result = JpegSegment::from_bytes(&bytes);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
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

                let result = JpegSegment::from_bytes(&bytes);
                assert_eq!(
                    result,
                    Ok(JpegSegment {
                        marker: JpegMarker::SOS,
                        length: Some(4),
                        data: Some(vec![
                            0x01, 0x02, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
                        ])
                    })
                );
            }
        }

        /// Tests for the `byte_count` function.
        mod test_byte_count {
            use super::*;

            ///Test when length and data are None
            #[test]
            fn test_no_data() {
                let segment = JpegSegment {
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
                    marker: JpegMarker::EOI,
                    length: None,
                    data: None,
                };

                assert_eq!(segment.as_bytes(), vec![0xFF, 0xD9]);
            }

            /// Test case for a segment that contains both the length and
            /// data sections.
            #[test]
            fn test_normal() {
                let segment = JpegSegment {
                    marker: JpegMarker::APP0,
                    length: Some(0x04),
                    data: Some(vec![0x01, 0x02]),
                };

                assert_eq!(segment.as_bytes(), vec![0xFF, 0xE0, 0x00, 0x04, 0x01, 0x02]);
            }
        }

        /// Test getting the segment as an XMP String
        #[test]
        fn test_as_xmp_str() {
            let data = "http://ns.adobe.com/xap/1.0/\0<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.1.0-jc003\"></x:xmpmeta>".as_bytes();
            let expected_str = String::from("<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.1.0-jc003\"></x:xmpmeta>");

            let segment = JpegSegment {
                marker: JpegMarker::APP1,
                length: Some(0x4EF),
                data: Some(Vec::from(data)),
            };

            assert_eq!(segment.as_xmp_str(), Some(expected_str));
        }

        /// Test trying to get non XMP segment as xmp string when the marker is wrong.
        #[test]
        fn test_as_xmp_str_wrong_marker() {
            let segment = JpegSegment {
                marker: JpegMarker::APP0,
                length: Some(0x04),
                data: Some(vec![0x01, 0x02, 0x03, 0x04]),
            };

            assert_eq!(segment.as_xmp_str(), None);
        }
        /// Test trying to get non XMP segment as xmp string when the contents is not right
        #[test]
        fn test_as_xmp_str_wrong_data() {
            let segment = JpegSegment {
                marker: JpegMarker::APP1,
                length: Some(0x04),
                data: Some(vec![0x01, 0x02, 0x03, 0x04]),
            };

            assert_eq!(segment.as_xmp_str(), None);
        }
    }
}
