/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Enumeration of errors the tool can produce.

use crate::jpeg::xmp::SemanticType;
use std::io::ErrorKind;
use thiserror::Error;

/// Enumeration of errors generated by the tool.
#[derive(PartialEq, Eq, Debug, Error)]
pub enum GCameraError {
    /// Indicates something went wrong with reading the image.
    #[error("Error reading the image. Kind: {kind}")]
    ImageReadError { kind: ErrorKind },

    /// Indicates something went wrong saving the image.
    #[error("Error writing the image. Kind: {kind}")]
    ImageWriteError { kind: ErrorKind },

    ///Indicates something went wrong saving the debug data
    #[error("Error writing the debug data. Kind: {kind}")]
    DebugDataWriteError { kind: ErrorKind },

    /// Indicates something went wro ng with saving the motion video
    #[error("Error writing motion video. Kind: {kind}")]
    MotionVideoWriteError { kind: ErrorKind },

    /// Indicates that the provided file does not have the correct magic bytes
    /// to be a JPEG file.
    #[error("File does not start with valid JPEG Magic.")]
    InvalidJpegMagic,

    /// Indicates that parsing the XML Document failed
    #[error("Error parsing XML Document. XML Error: {xml_error}.")]
    XMLParsingError {
        /// The XML Parser error
        #[source]
        xml_error: roxmltree::Error,
    },

    /// Indicates that an XML attribute could not be parsed
    #[error("Error parsing XML Attribute to a u32. Attribute: {attribute:?}.")]
    XMLAttributeParseError {
        /// The value of the attribute
        attribute: Option<String>,
    },

    /// Indicates that a required attribute could not be found.
    #[error("Required attribute '{attribute}' could not be found.")]
    XMLMissingAttribute {
        /// The attribute that could not be found.
        attribute: String,
    },

    /// Indicates that XMP Data could not be found in any segments.
    #[error("No XMP Data found in the image.")]
    NoXMPData,

    /// Indicates that the Description Node could not be found in the XML
    #[error("Description not found in XMP data.")]
    DescriptionNodeNotFound,

    /// Indicates that the given semantic string is not a known type.
    #[error("Resource has an unknown semantic type of '{semantic}'")]
    UnknownResourceSemantic {
        /// The string that could not be converted to a semantic enum.
        semantic: String,
    },

    /// Indicates that the given MIME Type is not known by the tool
    #[error("Resource has an unknown MIME Type of '{mime}'")]
    UnknownMimeType {
        /// The string that could not be converted to the Mimetype Enum
        mime: String,
    },

    /// Indicates that the next JPEG marker could not be found.
    #[error("Could not find another JPEG Segment Marker.")]
    JpegMarkerNotFound,

    /// Indicates that the type of JPEG Marker is not known.
    #[error("JPEG Marker with bytes '{marker_byte:02x}' is not known.")]
    UnknownJpegMarker {
        /// The bytes of the unknown marker.
        marker_byte: u8,
    },

    /// Indicates that the image contains no resources of the given type
    #[error("The image contains no resources of type {semantic_type:?}")]
    NoResourcesOfType {
        /// The type of resource that was searched for
        semantic_type: SemanticType,
    },
}
