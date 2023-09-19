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

#![warn(
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::restriction,
    clippy::cargo
)]
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
// Stuff from clippy:pedantic we don't worry about
#![allow(
    clippy::redundant_else,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools,
    clippy::match_same_arms,
    clippy::unreadable_literal,
    clippy::module_name_repetitions
)]
// Stuff from clippy::restriction we don't worry about
#![allow(
    clippy::question_mark_used,
    clippy::single_call_fn,
    clippy::missing_inline_in_public_items,
    clippy::use_debug,
    clippy::std_instead_of_alloc,
    clippy::print_stdout,
    clippy::std_instead_of_core,
    clippy::big_endian_bytes,
    clippy::pattern_type_mismatch,
    clippy::default_numeric_fallback,
    clippy::partial_pub_fields,
    clippy::indexing_slicing,
    clippy::blanket_clippy_restriction_lints,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::self_named_module_files
)]
// Stuff from clippy::restriction we might want to enable
#![allow(
    clippy::unwrap_in_result,
    clippy::map_err_ignore,
    clippy::exhaustive_enums, // TODO: What is this?
    clippy::exhaustive_structs, // TODO: What is this?
)]
// Stuff from clippy::restruction we do want.
#![allow(clippy::min_ident_chars)]
pub mod camera_image;
pub mod cli;
mod debug_components;
pub mod errors;
mod jpeg;
