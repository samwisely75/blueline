//! # REPL Entry Point
//!
//! This is the main entry point for the MVC-based REPL implementation.

use anyhow::Result;
use bluenote::IniProfile;

use crate::repl::controller::ReplController;

/// Create and run the MVC-based REPL
pub fn run(profile: IniProfile, verbose: bool) -> Result<()> {
    let mut controller = ReplController::new(profile, verbose)?;
    controller.run()
}
