//! Support for platform accessibility APIs via [AccessKit].
//!
//! This module provides the types and traits needed for widgets to declare
//! their semantics to assistive technologies (screen readers, etc.).
//!
//! [AccessKit]: https://accesskit.dev
pub use accesskit;

use crate::Rectangle;

/// Converts an iced [`Rectangle`] into an [`accesskit::Rect`].
pub fn rect(rectangle: Rectangle) -> accesskit::Rect {
    accesskit::Rect {
        x0: rectangle.x as f64,
        y0: rectangle.y as f64,
        x1: (rectangle.x + rectangle.width) as f64,
        y1: (rectangle.y + rectangle.height) as f64,
    }
}

/// Converts an [`accesskit::Rect`] into an iced [`Rectangle`].
pub fn from_rect(rect: accesskit::Rect) -> Rectangle {
    Rectangle {
        x: rect.x0 as f32,
        y: rect.y0 as f32,
        width: (rect.x1 - rect.x0) as f32,
        height: (rect.y1 - rect.y0) as f32,
    }
}
