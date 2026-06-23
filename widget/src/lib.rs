//! Use the built-in widgets or create your own.
//!
//! # Accessibility
//!
//! Iced has built-in support for assistive technologies (screen readers,
//! switch control, etc.) via the [AccessKit] library. All built-in widgets
//! expose their role, label, value, and available actions through the
//! accessibility tree, which is delivered to the platform accessibility API
//! each frame.
//!
//! ## Enabling accessibility
//!
//! Accessibility support is feature-gated behind the `accessibility` feature:
//!
//! ```toml
//! iced = { features = ["accessibility"] }
//! ```
//!
//! ## Adding accessibility to custom widgets
//!
//! To make a custom widget accessible, implement the [`accessibility`] method
//! on the [`Widget`] trait. The method receives the layout, widget tree, and a
//! mutable node buffer, and returns an optional [`NodeId`].
//!
//! ```ignore
//! #[cfg(feature = "accessibility")]
//! fn accessibility(
//!     &self,
//!     _layout: Layout<'_>,
//!     tree: &widget::Tree,
//!     nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
//!     id_counter: &mut u64,
//! ) -> Option<accesskit::NodeId> {
//!     let id = accesskit::NodeId(*id_counter);
//!     tree.set_accesskit_node_id(id);
//!     *id_counter += 1;
//!
//!     let mut node = accesskit::Node::new(accesskit::Role::Button);
//!     node.set_label("Click me");
//!     node.add_action(accesskit::Action::Click);
//!     nodes.push((id, node));
//!     Some(id)
//! }
//! ```
//!
//! You can also handle screen reader actions by implementing
//! [`accessibility_action`] on the [`Widget`] trait.
//!
//! ```ignore
//! #[cfg(feature = "accessibility")]
//! fn accessibility_action(
//!     &mut self,
//!     _tree: &mut widget::Tree,
//!     _layout: Layout<'_>,
//!     action: &accesskit::ActionRequest,
//!     shell: &mut Shell<'_, Message>,
//! ) {
//!     if action.action == accesskit::Action::Click {
//!         shell.publish(self.on_press.clone());
//!     }
//! }
//! ```
//!
//! See the documentation of the [`Widget`] trait for details.
//!
//! [AccessKit]: https://accesskit.dev/
//! [`accessibility`]: crate::core::Widget::accessibility
//! [`accessibility_action`]: crate::core::Widget::accessibility_action
//! [`Widget`]: crate::core::Widget
//! [`NodeId`]: accesskit::NodeId
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use iced_renderer as renderer;
pub use iced_renderer::core;
pub use iced_renderer::graphics;

pub use core::widget::{Id, Void};

mod action;
mod column;
mod mouse_area;
mod pin;
mod responsive;
mod stack;
mod themer;

pub mod button;
pub mod checkbox;
pub mod combo_box;
pub mod container;
pub mod float;
pub mod focus_ring;
pub mod grid;
pub mod keyed;
pub mod overlay;
pub mod pane_grid;
pub mod pick_list;
pub mod progress_bar;
pub mod radio;
pub mod row;
pub mod rule;
pub mod scrollable;
pub mod sensor;
pub mod slider;
pub mod space;
pub mod table;
pub mod text;
pub mod text_editor;
pub mod text_input;
pub mod toggler;
pub mod tooltip;
pub mod transition;
pub mod vertical_slider;

mod helpers;

pub use helpers::*;

#[cfg(feature = "lazy")]
mod lazy;

#[cfg(feature = "lazy")]
pub use crate::lazy::helpers::*;

#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use checkbox::Checkbox;
#[doc(no_inline)]
pub use column::Column;
#[doc(no_inline)]
pub use combo_box::ComboBox;
#[doc(no_inline)]
pub use container::Container;
#[doc(no_inline)]
pub use float::Float;
#[doc(no_inline)]
pub use grid::Grid;
#[doc(no_inline)]
pub use mouse_area::MouseArea;
#[doc(no_inline)]
pub use pane_grid::PaneGrid;
#[doc(no_inline)]
pub use pick_list::PickList;
#[doc(no_inline)]
pub use pin::Pin;
#[doc(no_inline)]
pub use progress_bar::ProgressBar;
#[doc(no_inline)]
pub use radio::Radio;
#[doc(no_inline)]
pub use responsive::Responsive;
#[doc(no_inline)]
pub use row::Row;
#[doc(no_inline)]
pub use rule::Rule;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use sensor::Sensor;
#[doc(no_inline)]
pub use slider::Slider;
#[doc(no_inline)]
pub use space::Space;
#[doc(no_inline)]
pub use stack::Stack;
#[doc(no_inline)]
pub use text::Text;
#[doc(no_inline)]
pub use text_editor::TextEditor;
#[doc(no_inline)]
pub use text_input::TextInput;
#[doc(no_inline)]
pub use themer::Themer;
#[doc(no_inline)]
pub use toggler::Toggler;
#[doc(no_inline)]
pub use tooltip::Tooltip;
#[doc(no_inline)]
pub use vertical_slider::VerticalSlider;

#[cfg(feature = "wgpu")]
pub mod shader;

#[cfg(feature = "wgpu")]
#[doc(no_inline)]
pub use shader::Shader;

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "svg")]
#[doc(no_inline)]
pub use svg::Svg;

#[cfg(feature = "image")]
pub mod image;

#[cfg(feature = "image")]
#[doc(no_inline)]
pub use image::Image;

#[cfg(feature = "canvas")]
pub mod canvas;

#[cfg(feature = "canvas")]
#[doc(no_inline)]
pub use canvas::Canvas;

#[cfg(feature = "qr_code")]
pub mod qr_code;

#[cfg(feature = "qr_code")]
#[doc(no_inline)]
pub use qr_code::QRCode;

#[cfg(feature = "markdown")]
pub mod markdown;

pub use crate::core::theme::{self, Theme};
pub use action::Action;
pub use renderer::Renderer;
