//! Containers let you align a widget inside their boundaries.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::container;
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     container("This text is centered inside a rounded box!")
//!         .padding(10)
//!         .center(800)
//!         .style(container::rounded_box)
//!         .into()
//! }
//! ```
use crate::core::alignment::{self, Alignment};
use crate::core::border::{self, Border};
use crate::core::gradient::{self, Gradient};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::theme;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Operation};
use crate::core::{
    self, Background, Color, Element, Event, Layout, Length, Padding, Rectangle, Shadow, Shell,
    Size, Theme, Vector, Widget, color,
};

/// A widget that aligns its contents inside of its boundaries.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::container;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     container("This text is centered inside a rounded box!")
///         .padding(10)
///         .center(800)
///         .style(container::rounded_box)
///         .into()
/// }
/// ```
pub struct Container<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    id: Option<widget::Id>,
    accessible_label: Option<String>,
    focusable: bool,
    padding: Padding,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    clip: bool,
    content: Element<'a, Message, Theme, Renderer>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Container<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    /// Creates a [`Container`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();

        Container {
            id: None,
            accessible_label: None,
            focusable: false,
            padding: Padding::ZERO,
            width: Length::Fit,
            height: Length::Fit,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            clip: false,
            class: Theme::default(),
            content,
        }
    }

    /// Sets the [`widget::Id`] of the [`Container`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the accessible label of the [`Container`].
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Enables the [`Container`] to be focused via keyboard navigation.
    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Container`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the width of the [`Container`] and centers its contents horizontally.
    pub fn center_x(self, width: impl Into<Length>) -> Self {
        self.width(width).align_x(alignment::Horizontal::Center)
    }

    /// Sets the height of the [`Container`] and centers its contents vertically.
    pub fn center_y(self, height: impl Into<Length>) -> Self {
        self.height(height).align_y(alignment::Vertical::Center)
    }

    /// Sets the width and height of the [`Container`] and centers its contents in
    /// both the horizontal and vertical axes.
    ///
    /// This is equivalent to chaining [`center_x`] and [`center_y`].
    ///
    /// [`center_x`]: Self::center_x
    /// [`center_y`]: Self::center_y
    pub fn center(self, length: impl Into<Length>) -> Self {
        let length = length.into();

        self.center_x(length).center_y(length)
    }

    /// Sets the width of the [`Container`] and aligns its contents to the left.
    pub fn align_left(self, width: impl Into<Length>) -> Self {
        self.width(width).align_x(alignment::Horizontal::Left)
    }

    /// Sets the width of the [`Container`] and aligns its contents to the right.
    pub fn align_right(self, width: impl Into<Length>) -> Self {
        self.width(width).align_x(alignment::Horizontal::Right)
    }

    /// Sets the height of the [`Container`] and aligns its contents to the top.
    pub fn align_top(self, height: impl Into<Length>) -> Self {
        self.height(height).align_y(alignment::Vertical::Top)
    }

    /// Sets the height of the [`Container`] and aligns its contents to the bottom.
    pub fn align_bottom(self, height: impl Into<Length>) -> Self {
        self.height(height).align_y(alignment::Vertical::Bottom)
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.horizontal_alignment = alignment.into();
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.vertical_alignment = alignment.into();
        self
    }

    /// Sets whether the contents of the [`Container`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Container`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Container`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Container<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        if self.focusable {
            crate::focus_ring::FocusState::tag()
        } else {
            self.content.as_widget().tag()
        }
    }

    fn state(&self) -> tree::State {
        if self.focusable {
            crate::focus_ring::FocusState::state()
        } else {
            self.content.as_widget().state()
        }
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));

        let size = self.content.as_widget().size();
        self.width = self.width.stack(size.width);
        self.height = self.height.stack(size.height);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            limits,
            self.width,
            self.height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |limits| {
                self.content
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.focusable {
            let state = tree.state.downcast_mut::<crate::focus_ring::FocusState>();
            operation.focusable(self.id.as_ref(), layout.bounds(), state);
        }
        operation.container(self.id.as_ref(), layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    #[cfg(feature = "accessibility")]
    fn accessibility(
        &self,
        layout: crate::core::Layout<'_>,
        tree: &crate::core::widget::Tree,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        id_counter: &mut u64,
    ) -> Option<accesskit::NodeId> {
        // Forward child accessibility first
        let child_id = self.content.as_widget().accessibility(
            layout.children().next().unwrap(),
            &tree.children[0],
            nodes,
            id_counter,
        );

        let id = accesskit::NodeId(*id_counter);
        tree.set_accesskit_node_id(id);
        *id_counter += 1;

        let role = if self.accessible_label.is_some() {
            accesskit::Role::Group
        } else {
            accesskit::Role::GenericContainer
        };
        let mut builder = accesskit::Node::new(role);
        crate::core::accessibility::set_bounds(tree, &mut builder, layout.bounds());

        if let Some(child_id) = child_id {
            builder.push_child(child_id);
        }

        if let Some(label) = &self.accessible_label {
            builder.set_label(label.as_str());
        }

        if self.focusable {
            builder.add_action(accesskit::Action::Focus);
            builder.add_child_action(accesskit::Action::Focus);

            if tree
                .state
                .downcast_ref::<crate::focus_ring::FocusState>()
                .is_focused
            {
                tree.set_accesskit_focused(true);
            }
        }

        nodes.push((id, builder));

        Some(id)
    }

    #[cfg(feature = "accessibility")]
    fn accessibility_action(
        &mut self,
        tree: &mut crate::core::widget::Tree,
        layout: crate::core::Layout<'_>,
        action: &accesskit::ActionRequest,
        shell: &mut crate::core::Shell<'_, Message>,
    ) {
        if self.focusable && tree.owns_accesskit_node_id(action.target_node) {
            if action.action == accesskit::Action::Focus {
                tree.state
                    .downcast_mut::<crate::focus_ring::FocusState>()
                    .is_focused = true;
                shell.request_redraw();
            }
            return;
        }

        // Forward to children
        if let (Some(state), Some(layout)) = (tree.children.first_mut(), layout.children().next()) {
            if !state.contains_accesskit_node_id(action.target_node) {
                return;
            }

            self.content
                .as_widget_mut()
                .accessibility_action(state, layout, action, shell);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        #[cfg(feature = "accessibility")]
        if self.focusable && tree.accesskit_focused() {
            crate::focus_ring::draw(renderer, bounds, &crate::focus_ring::Appearance::default());
        }

        let style = theme.style(&self.class);

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            draw_background(renderer, &style, bounds);

            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: style.text_color.unwrap_or(renderer_style.text_color),
                },
                layout.children().next().unwrap(),
                cursor,
                if self.clip {
                    &clipped_viewport
                } else {
                    viewport
                },
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Container<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(
        container: Container<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(container)
    }
}

/// Computes the layout of a [`Container`].
pub fn layout(
    limits: &layout::Limits,
    width: Length,
    height: Length,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    layout_content: impl FnOnce(&layout::Limits) -> layout::Node,
) -> layout::Node {
    layout::positioned(
        limits,
        width,
        height,
        padding,
        |limits| layout_content(&limits.loose()),
        |content, size| {
            content.align(
                Alignment::from(horizontal_alignment),
                Alignment::from(vertical_alignment),
                size,
            )
        },
    )
}

/// Draws the background of a [`Container`] given its [`Style`] and its `bounds`.
pub fn draw_background<Renderer>(renderer: &mut Renderer, style: &Style, bounds: Rectangle)
where
    Renderer: core::Renderer,
{
    if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                shadow: style.shadow,
                snap: style.snap,
            },
            style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }
}

/// The appearance of a container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The text [`Color`] of the container.
    pub text_color: Option<Color>,
    /// The [`Background`] of the container.
    pub background: Option<Background>,
    /// The [`Border`] of the container.
    pub border: Border,
    /// The [`Shadow`] of the container.
    pub shadow: Shadow,
    /// Whether the container should be snapped to the pixel grid.
    pub snap: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: renderer::CRISP,
        }
    }
}

impl Style {
    /// Updates the text color of the [`Style`].
    pub fn color(self, color: impl Into<Color>) -> Self {
        Self {
            text_color: Some(color.into()),
            ..self
        }
    }

    /// Updates the border of the [`Style`].
    pub fn border(self, border: impl Into<Border>) -> Self {
        Self {
            border: border.into(),
            ..self
        }
    }

    /// Updates the background of the [`Style`].
    pub fn background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }

    /// Updates the shadow of the [`Style`].
    pub fn shadow(self, shadow: impl Into<Shadow>) -> Self {
        Self {
            shadow: shadow.into(),
            ..self
        }
    }
}

impl From<Color> for Style {
    fn from(color: Color) -> Self {
        Self::default().background(color)
    }
}

impl From<Gradient> for Style {
    fn from(gradient: Gradient) -> Self {
        Self::default().background(gradient)
    }
}

impl From<gradient::Linear> for Style {
    fn from(gradient: gradient::Linear) -> Self {
        Self::default().background(gradient)
    }
}

/// The theme catalog of a [`Container`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Container`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl<Theme> From<Style> for StyleFn<'_, Theme> {
    fn from(style: Style) -> Self {
        Box::new(move |_theme| style)
    }
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// A transparent [`Container`].
pub fn transparent<Theme>(_theme: &Theme) -> Style {
    Style::default()
}

/// A [`Container`] with the given [`Background`].
pub fn background(background: impl Into<Background>) -> Style {
    Style::default().background(background)
}

/// A rounded [`Container`] with a background.
pub fn rounded_box(theme: &Theme) -> Style {
    let palette = theme.palette();

    Style {
        background: Some(palette.background.weak.color.into()),
        text_color: Some(palette.background.weak.text),
        border: border::rounded(2),
        ..Style::default()
    }
}

/// A bordered [`Container`] with a background.
pub fn bordered_box(theme: &Theme) -> Style {
    let palette = theme.palette();

    Style {
        background: Some(palette.background.weakest.color.into()),
        text_color: Some(palette.background.weakest.text),
        border: Border {
            width: 1.0,
            radius: 5.0.into(),
            color: palette.background.weak.color,
        },
        ..Style::default()
    }
}

/// A [`Container`] with a dark background and white text.
pub fn dark(_theme: &Theme) -> Style {
    style(theme::palette::Pair {
        color: color!(0x111111),
        text: Color::WHITE,
    })
}

/// A [`Container`] with a primary background color.
pub fn primary(theme: &Theme) -> Style {
    let palette = theme.palette();

    style(palette.primary.base)
}

/// A [`Container`] with a secondary background color.
pub fn secondary(theme: &Theme) -> Style {
    let palette = theme.palette();

    style(palette.secondary.base)
}

/// A [`Container`] with a success background color.
pub fn success(theme: &Theme) -> Style {
    let palette = theme.palette();

    style(palette.success.base)
}

/// A [`Container`] with a warning background color.
pub fn warning(theme: &Theme) -> Style {
    let palette = theme.palette();

    style(palette.warning.base)
}

/// A [`Container`] with a danger background color.
pub fn danger(theme: &Theme) -> Style {
    let palette = theme.palette();

    style(palette.danger.base)
}

fn style(pair: theme::palette::Pair) -> Style {
    Style {
        background: Some(pair.color.into()),
        text_color: Some(pair.text),
        border: border::rounded(2),
        ..Style::default()
    }
}
