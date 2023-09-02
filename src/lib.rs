/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Functionality for working with with photos taken with Google Camera
//!
//! Photographs taken with Google camera often contain additional
//! metadata or images that are not part of a normal JPEG image.
//! The goal of the crate is to provide functionality for identifying
//! this additional data, and either extracting it from the image, or
//! removing it altogether. This is provided both through a library, and
//! as a command line tool.

pub mod arguments;
pub mod camera_image;
pub mod debug_components;
pub mod jpeg_components;
pub mod xmp;
