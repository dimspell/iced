//! Show focus rings around focused widgets.
use crate::core::{Background, Border, Color, Rectangle, Renderer};

/// The appearance of a focus ring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    /// The color of the focus ring.
    pub color: Color,
    /// The width of the focus ring border, in logical pixels.
    pub width: f32,
    /// The gap between the widget bounds and the focus ring, in logical pixels.
    pub offset: f32,
    /// The border radius of the focus ring.
    pub border_radius: crate::core::border::Radius,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            color: Color::from_rgb(0.0, 0.5, 1.0),
            width: 2.0,
            offset: 2.0,
            border_radius: crate::core::border::Radius::default(),
        }
    }
}

/// A styling trait for focus rings.
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Appearance`] of the focus ring for the given class.
    fn focus_ring(&self, class: &Self::Class<'_>) -> Appearance;
}

/// A styling function for a [`FocusRing`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Appearance + 'a>;

/// Draws a focus ring around the given bounds.
///
/// The focus ring is drawn as a colored halo slightly larger than the widget
/// bounds. The widget draws its own background on top, which covers the inner
/// portion and leaves only the ring visible.
pub fn draw<R: Renderer>(
    renderer: &mut R,
    bounds: Rectangle,
    appearance: &Appearance,
) {
    use crate::core::renderer::Quad;

    let ring_bounds = Rectangle {
        x: bounds.x - appearance.offset,
        y: bounds.y - appearance.offset,
        width: bounds.width + 2.0 * appearance.offset,
        height: bounds.height + 2.0 * appearance.offset,
    };

    renderer.fill_quad(
        Quad {
            bounds: ring_bounds,
            border: Border {
                color: appearance.color,
                width: appearance.width,
                radius: appearance.border_radius,
            },
            ..Default::default()
        },
        Background::Color(appearance.color),
    );
}

/// Default focus ring style.
pub fn default(theme: &crate::Theme) -> Appearance {
    let palette = theme.palette();

    Appearance {
        color: palette.primary.base.color,
        ..Default::default()
    }
}

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn focus_ring(&self, class: &Self::Class<'_>) -> Appearance {
        class(self)
    }
}
