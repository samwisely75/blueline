//! # New REPL Entry Point
//!
//! This is the new MVC-based REPL implementation that will eventually replace
//! the monolithic repl.rs. This serves as a bridge during the transition.

use anyhow::Result;
use bluenote::IniProfile;

use crate::repl::controller::ReplController;

/// Create and run the new MVC-based REPL
pub async fn run_new_repl(profile: IniProfile, verbose: bool) -> Result<()> {
    let mut controller = ReplController::new(profile, verbose)?;
    controller.run().await
}
