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
