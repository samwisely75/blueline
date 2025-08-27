//! Visual Block Insert cursor management for PaneState
//!
//! This module contains methods for managing multi-cursor state during Visual Block Insert mode.

use crate::repl::events::LogicalPosition;

use super::PaneState;

impl PaneState {
    /// Set Visual Block Insert cursor positions
    /// This also sets the initial boundary columns (only on first call)
    pub fn set_visual_block_insert_cursors(&mut self, positions: Vec<LogicalPosition>) {
        self.visual_block_insert_cursors = positions;

        // Only set start columns if they're not already set (preserve boundaries)
        if self.visual_block_insert_start_columns.is_empty() {
            // Store the start columns as boundaries - extract column from each position
            self.visual_block_insert_start_columns = self
                .visual_block_insert_cursors
                .iter()
                .map(|pos| pos.column)
                .collect();
            tracing::debug!(
                "Set {} Visual Block Insert cursor positions with initial start columns: {:?}",
                self.visual_block_insert_cursors.len(),
                self.visual_block_insert_start_columns
            );
        } else {
            tracing::debug!(
                "Updated {} Visual Block Insert cursor positions, preserving start columns: {:?}",
                self.visual_block_insert_cursors.len(),
                self.visual_block_insert_start_columns
            );
        }
    }

    /// Update only the cursor positions without changing boundaries
    pub fn update_visual_block_insert_cursors(&mut self, positions: Vec<LogicalPosition>) {
        self.visual_block_insert_cursors = positions;
        tracing::debug!(
            "Updated {} Visual Block Insert cursor positions, preserving boundaries: {:?}",
            self.visual_block_insert_cursors.len(),
            self.visual_block_insert_start_columns
        );
    }

    /// Get Visual Block Insert cursor positions  
    pub fn get_visual_block_insert_cursors(&self) -> &[LogicalPosition] {
        &self.visual_block_insert_cursors
    }

    /// Get Visual Block Insert start column boundaries
    pub fn get_visual_block_insert_start_columns(&self) -> &[usize] {
        &self.visual_block_insert_start_columns
    }

    /// Clear Visual Block Insert cursor positions
    pub fn clear_visual_block_insert_cursors(&mut self) {
        self.visual_block_insert_cursors.clear();
        self.visual_block_insert_start_columns.clear();
        tracing::debug!("Cleared Visual Block Insert cursor positions and boundaries");
    }

    /// Check if in Visual Block Insert mode (has multi-cursors)
    pub fn is_in_visual_block_insert_mode(&self) -> bool {
        !self.visual_block_insert_cursors.is_empty()
    }
}
