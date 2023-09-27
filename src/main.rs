/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
use gcamera_tools::cli::tool::tool_main;
use std::process::exit;

/// Main function that is run from the command line.
fn main() {
    tool_main().unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        exit(1)
    });
}
