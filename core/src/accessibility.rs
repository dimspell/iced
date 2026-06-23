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

/// Returns the smallest rectangle containing both rectangles.
pub fn union(a: Rectangle, b: Rectangle) -> Rectangle {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.width).max(b.x + b.width);
    let y1 = (a.y + a.height).max(b.y + b.height);

    Rectangle {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    }
}

/// Ensures a rectangle has a non-empty size.
pub fn non_empty(rectangle: Rectangle) -> Rectangle {
    Rectangle {
        width: rectangle.width.max(1.0),
        height: rectangle.height.max(1.0),
        ..rectangle
    }
}
