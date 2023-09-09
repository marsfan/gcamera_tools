/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Enumeration of errors the tool can produce.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

/// Enumeration of errors generated by the tool.
#[derive(PartialEq, Eq, Debug)]
pub enum GCameraError {
    /// Indicates something went wrong with reading the image.
    // TODO: Encapsulate the std::io::Error that was the source?
    ImageReadError,

    /// Indicates something went wrong saving the image.
    // TODO: Encapsulate the std::io::Error that was the source?
    ImageWriteError,

    ///Indicates something went wrong saving the debug data
    // TODO: Encapsulate the std::io::Error that was the source?
    DebugDataWriteError,

    /// Catch-all for any other possible error type
    Other {
        /// The error message
        msg: String,
    },
}

/// Implementation to automatically convert Err<&'static str> into GCameraError::Other
impl From<&str> for GCameraError {
    /// Convert string slice to GCameraError::Other
    ///
    /// # Arguments
    /// * `val`: The value to convert
    ///
    /// # Returns
    /// GCameraError::Other with the input string as the message field
    fn from(val: &str) -> Self {
        return Self::Other {
            msg: String::from(val),
        };
    }
}