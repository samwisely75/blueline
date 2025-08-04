//! # Geometry Types
//!
//! Provides reusable geometry types to replace tuple usage throughout the codebase.
//! This module contains Position and Dimensions structs with proper semantic naming
//! and convenience methods.

/// A position in 2D space with row and column coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    /// Create a new position
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Create a position at the origin (0, 0)
    pub const fn origin() -> Self {
        Self::new(0, 0)
    }
}

/// Dimensions representing width and height
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    /// Create new dimensions
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Create zero dimensions
    pub const fn zero() -> Self {
        Self::new(0, 0)
    }

    /// Get the area (width * height)
    pub const fn area(self) -> usize {
        self.width * self.height
    }

    /// Check if dimensions are empty (width or height is 0)
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.row, 5);
        assert_eq!(pos.col, 10);
    }

    #[test]
    fn test_position_origin() {
        let pos = Position::origin();
        assert_eq!(pos, Position::new(0, 0));
    }

    #[test]
    fn test_dimensions_creation() {
        let dims = Dimensions::new(80, 24);
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
    }

    #[test]
    fn test_dimensions_zero() {
        let dims = Dimensions::zero();
        assert_eq!(dims, Dimensions::new(0, 0));
        assert!(dims.is_empty());
    }

    #[test]
    fn test_dimensions_area() {
        let dims = Dimensions::new(10, 20);
        assert_eq!(dims.area(), 200);
    }

    #[test]
    fn test_dimensions_is_empty() {
        assert!(Dimensions::new(0, 10).is_empty());
        assert!(Dimensions::new(10, 0).is_empty());
        assert!(!Dimensions::new(10, 20).is_empty());
    }
}
