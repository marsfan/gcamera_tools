/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Logic for handling JPEG markers.

use crate::errors::GCameraError;

/// Enum of the different JPEG segment markers.
#[allow(clippy::upper_case_acronyms)] // Allowing because names are upper for JPEG segments
#[allow(clippy::missing_docs_in_private_items)] // Allowing since documenting this would be a pain
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
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
        return value as u8;
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

#[cfg(test)]
mod tests {
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
