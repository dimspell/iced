use crate::container;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{self, Tree};
use crate::core::{self, Element, Event, Layout, Point, Rectangle, Shell, Size, Vector};
use crate::pane_grid::{Draggable, TitleBar};

/// The content of a [`Pane`].
///
/// [`Pane`]: super::Pane
pub struct Content<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    title_bar: Option<TitleBar<'a, Message, Theme, Renderer>>,
    body: Element<'a, Message, Theme, Renderer>,
    class: Theme::Class<'a>,
    /// The accessible label for this pane, if any.
    #[cfg(feature = "accessibility")]
    accessible_label: Option<String>,
}

impl<'a, Message, Theme, Renderer> Content<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    /// Creates a new [`Content`] with the provided body.
    pub fn new(body: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            title_bar: None,
            body: body.into(),
            class: Theme::default(),
            #[cfg(feature = "accessibility")]
            accessible_label: None,
        }
    }

    /// Sets the [`TitleBar`] of the [`Content`].
    pub fn title_bar(mut self, title_bar: TitleBar<'a, Message, Theme, Renderer>) -> Self {
        self.title_bar = Some(title_bar);
        self
    }

    /// Sets the style of the [`Content`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<container::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as container::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Content`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the accessible label for this pane, enabling screen readers
    /// to identify the pane by name.
    ///
    /// In a tiling-window-manager-style UI, each pane should have a
    /// distinguishable label (e.g., "Editor", "Terminal", "File Explorer").
    #[cfg(feature = "accessibility")]
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
}

impl<Message, Theme, Renderer> Content<'_, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    #[cfg(feature = "accessibility")]
    pub(super) fn accessibility(
        &self,
        layout: Layout<'_>,
        tree: &Tree,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        id_counter: &mut u64,
    ) -> Option<accesskit::NodeId> {
        use crate::core::accessibility::accesskit;

        // Body is always at tree.children[0]
        let body_layout = if self.title_bar.is_some() {
            layout.children().nth(1)
        } else {
            Some(layout)
        };

        let body_id = body_layout.and_then(|body_layout| {
            self.body
                .as_widget()
                .accessibility(body_layout, &tree.children[0], nodes, id_counter)
        })?;

        // If a pane label is set, wrap the body in a labeled container
        // so screen readers can identify individual panes by name.
        if let Some(label) = &self.accessible_label {
            let id = accesskit::NodeId(*id_counter);
            *id_counter += 1;

            let mut builder =
                accesskit::Node::new(accesskit::Role::GenericContainer);
            builder.push_child(body_id);
            builder.set_label(label.as_str());
            nodes.push((id, builder));

            Some(id)
        } else {
            Some(body_id)
        }
    }

    #[cfg(feature = "accessibility")]
    pub(super) fn accessibility_action(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        action: &accesskit::ActionRequest,
        shell: &mut Shell<'_, Message>,
    ) {
        let body_layout = if self.title_bar.is_some() {
            layout.children().nth(1)
        } else {
            Some(layout)
        };

        if let Some(body_layout) = body_layout {
            self.body
                .as_widget_mut()
                .accessibility_action(&mut tree.children[0], body_layout, action, shell);
        }
    }

    pub(super) fn state(&self) -> Tree {
        let children = if let Some(title_bar) = self.title_bar.as_ref() {
            vec![Tree::new(&self.body), title_bar.state()]
        } else {
            vec![Tree::new(&self.body), Tree::empty()]
        };

        Tree {
            children,
            ..Tree::empty()
        }
    }

    pub(super) fn diff(&mut self, tree: &mut Tree) {
        if tree.children.len() == 2 {
            if let Some(title_bar) = &mut self.title_bar {
                title_bar.diff(&mut tree.children[1]);
            }

            tree.children[0].diff(&mut self.body);
        } else {
            *tree = self.state();
        }
    }

    /// Draws the [`Content`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: core::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        {
            let style = theme.style(&self.class);

            container::draw_background(renderer, &style, bounds);
        }

        if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();
            let body_layout = children.next().unwrap();

            let show_controls = cursor.is_over(bounds);

            self.body.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                body_layout,
                cursor,
                viewport,
            );

            title_bar.draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                title_bar_layout,
                cursor,
                viewport,
                show_controls,
            );
        } else {
            self.body.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
    }

    pub(crate) fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if let Some(title_bar) = &mut self.title_bar {
            let max_size = limits.max();

            let title_bar_layout = title_bar.layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(Size::ZERO, max_size),
            );

            let title_bar_size = title_bar_layout.size();

            let body_layout = self.body.as_widget_mut().layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    Size::new(max_size.width, max_size.height - title_bar_size.height),
                ),
            );

            layout::Node::with_children(
                max_size,
                vec![
                    title_bar_layout,
                    body_layout.move_to(Point::new(0.0, title_bar_size.height)),
                ],
            )
        } else {
            self.body
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        }
    }

    pub(crate) fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let body_layout = if let Some(title_bar) = &mut self.title_bar {
            let mut children = layout.children();

            title_bar.operate(
                &mut tree.children[1],
                children.next().unwrap(),
                renderer,
                operation,
            );

            children.next().unwrap()
        } else {
            layout
        };

        self.body
            .as_widget_mut()
            .operate(&mut tree.children[0], body_layout, renderer, operation);
    }

    pub(crate) fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
        is_picked: bool,
    ) {
        let body_layout = if let Some(title_bar) = &mut self.title_bar {
            let mut children = layout.children();

            title_bar.update(
                &mut tree.children[1],
                event,
                children.next().unwrap(),
                cursor,
                renderer,
                shell,
                viewport,
            );

            children.next().unwrap()
        } else {
            layout
        };

        if !is_picked {
            self.body.as_widget_mut().update(
                &mut tree.children[0],
                event,
                body_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }
    }

    pub(crate) fn grid_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        drag_enabled: bool,
    ) -> Option<mouse::Interaction> {
        let title_bar = self.title_bar.as_ref()?;

        let mut children = layout.children();
        let title_bar_layout = children.next().unwrap();

        let is_over_pick_area = cursor
            .position()
            .map(|cursor_position| title_bar.is_over_pick_area(title_bar_layout, cursor_position))
            .unwrap_or_default();

        if is_over_pick_area && drag_enabled {
            return Some(mouse::Interaction::Grab);
        }

        None
    }

    pub(crate) fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
        drag_enabled: bool,
    ) -> mouse::Interaction {
        let (body_layout, title_bar_interaction) = if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();

            let is_over_pick_area = cursor
                .position()
                .map(|cursor_position| {
                    title_bar.is_over_pick_area(title_bar_layout, cursor_position)
                })
                .unwrap_or_default();

            if is_over_pick_area && drag_enabled {
                return mouse::Interaction::Grab;
            }

            let mouse_interaction = title_bar.mouse_interaction(
                &tree.children[1],
                title_bar_layout,
                cursor,
                viewport,
                renderer,
            );

            (children.next().unwrap(), mouse_interaction)
        } else {
            (layout, mouse::Interaction::default())
        };

        self.body
            .as_widget()
            .mouse_interaction(&tree.children[0], body_layout, cursor, viewport, renderer)
            .max(title_bar_interaction)
    }

    pub(crate) fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if let Some(title_bar) = self.title_bar.as_mut() {
            let mut children = layout.children();
            let title_bar_layout = children.next()?;

            let mut states = tree.children.iter_mut();
            let body_state = states.next().unwrap();
            let title_bar_state = states.next().unwrap();

            match title_bar.overlay(
                title_bar_state,
                title_bar_layout,
                renderer,
                viewport,
                translation,
            ) {
                Some(overlay) => Some(overlay),
                None => self.body.as_widget_mut().overlay(
                    body_state,
                    children.next()?,
                    renderer,
                    viewport,
                    translation,
                ),
            }
        } else {
            self.body.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }
}

impl<Message, Theme, Renderer> Draggable for &Content<'_, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    fn can_be_dragged_at(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
        if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();

            title_bar.is_over_pick_area(title_bar_layout, cursor_position)
        } else {
            false
        }
    }
}

impl<'a, T, Message, Theme, Renderer> From<T> for Content<'a, Message, Theme, Renderer>
where
    T: Into<Element<'a, Message, Theme, Renderer>>,
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    fn from(element: T) -> Self {
        Self::new(element)
    }
}
