//! Implement your own event loop to drive a user interface.
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::shell;
use crate::core::widget;
use crate::core::window;
use crate::core::{
    Clipboard, Element, InputMethod, Layout, Rectangle, Shell, Size, Vector, Window,
};

#[cfg(feature = "accessibility")]
use crate::core::accessibility::accesskit;

/// A set of interactive graphical elements with a specific [`Layout`].
///
/// It can be updated and drawn.
///
/// Iced tries to avoid dictating how to write your event loop. You are in
/// charge of using this type in your system in any way you want.
///
/// # Example
/// The [`integration`] example uses a [`UserInterface`] to integrate Iced in an
/// existing graphical application.
///
/// [`integration`]: https://github.com/iced-rs/iced/tree/master/examples/integration
pub struct UserInterface<'a, Message, Theme, Renderer> {
    root: Element<'a, Message, Theme, Renderer>,
    base: layout::Node,
    state: widget::Tree,
    overlay: Option<Overlay>,
    bounds: Size,
}

struct Overlay {
    layout: layout::Node,
    interaction: mouse::Interaction,
}

impl<'a, Message, Theme, Renderer> UserInterface<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Builds a user interface for an [`Element`].
    ///
    /// It is able to avoid expensive computations when using a [`Cache`]
    /// obtained from a previous instance of a [`UserInterface`].
    ///
    /// # Example
    /// Imagine we want to build a [`UserInterface`] for
    /// [the counter example that we previously wrote](index.html#usage). Here
    /// is naive way to set up our application loop:
    ///
    /// ```no_run
    /// # mod iced_wgpu {
    /// #     pub type Renderer = ();
    /// # }
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> iced_core::Element<(), (), Renderer> { unimplemented!() }
    /// #     pub fn update(&mut self, _: ()) {}
    /// # }
    /// use iced_runtime::core::shell;
    /// use iced_runtime::core::window;
    /// use iced_runtime::core::Size;
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// // Initialization
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window = window::Headless; // This should be a proper window, like a `winit` one
    /// let mut waker = shell::Waker::noop();
    /// let mut window_size = Size::new(1024.0, 768.0);
    ///
    /// // Application loop
    /// loop {
    ///     // Process system events here...
    ///
    ///     // Build the user interface
    ///     let user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     // Update and draw the user interface here...
    ///     // ...
    ///
    ///     // Obtain the cache for the next iteration
    ///     cache = user_interface.into_cache();
    /// }
    /// ```
    pub fn build<E: Into<Element<'a, Message, Theme, Renderer>>>(
        root: E,
        bounds: Size,
        cache: Cache,
        renderer: &mut Renderer,
    ) -> Self {
        let mut root = root.into();

        let Cache { mut state } = cache;
        state.diff(root.as_widget_mut());

        let base = root.as_widget_mut().layout(
            &mut state,
            renderer,
            &layout::Limits::new(Size::ZERO, bounds),
        );

        UserInterface {
            root,
            base,
            state,
            overlay: None,
            bounds,
        }
    }

    /// Updates the [`UserInterface`] by processing each provided [`Event`].
    ///
    /// It returns __messages__ that may have been produced as a result of user
    /// interactions. You should feed these to your __update logic__.
    ///
    /// # Example
    /// Let's allow our [counter](index.html#usage) to change state by
    /// completing [the previous example](#example):
    ///
    /// ```no_run
    /// # mod iced_wgpu {
    /// #     pub type Renderer = ();
    /// # }
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> iced_core::Element<(), (), Renderer> { unimplemented!() }
    /// #     pub fn update(&mut self, _: ()) {}
    /// # }
    /// use iced_runtime::core::mouse;
    /// use iced_runtime::core::shell;
    /// use iced_runtime::core::window;
    /// use iced_runtime::core::Size;
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window = window::Headless; // This should be a proper window, like a `winit` one
    /// let mut waker = shell::Waker::noop();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor = mouse::Cursor::default();
    ///
    /// // Initialize our event storage
    /// let mut events = Vec::new();
    /// let mut messages = shell::Bus::new();
    ///
    /// loop {
    ///     // Obtain system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     // Update the user interface
    ///     let (state, event_statuses) = user_interface.update(
    ///         &window,
    ///         &waker,
    ///         &events,
    ///         cursor,
    ///         &mut renderer,
    ///         &mut messages
    ///     );
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     // Process the produced messages
    ///     for message in messages.drain() {
    ///         counter.update(message);
    ///     }
    /// }
    /// ```
    pub fn update(
        &mut self,
        window: &dyn Window,
        waker: &shell::Waker,
        events: &[Event],
        cursor: mouse::Cursor,
        renderer: &mut Renderer,
        messages: &mut shell::Bus<Message>,
    ) -> (State, Vec<event::Status>) {
        let mut outdated = false;
        let mut redraw_request = window::RedrawRequest::Wait;
        let mut input_method = InputMethod::Disabled;
        let mut clipboard = Clipboard::new();
        let mut has_layout_changed = false;
        let viewport = Rectangle::with_size(self.bounds);

        let mut maybe_overlay = self
            .root
            .as_widget_mut()
            .overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
                &viewport,
                Vector::ZERO,
            )
            .map(overlay::Nested::new);

        let (base_cursor, overlay_statuses, overlay_interaction) = if maybe_overlay.is_some() {
            let bounds = self.bounds;

            let mut overlay = maybe_overlay.as_mut().unwrap();
            let mut layout = overlay.layout(renderer, bounds);
            let mut event_statuses = Vec::new();

            for event in events {
                let mut shell = Shell::new(window, waker.clone(), messages);

                overlay.update(event, Layout::new(&layout), cursor, renderer, &mut shell);

                event_statuses.push(shell.event_status());
                redraw_request = redraw_request.min(shell.redraw_request());
                input_method.merge(shell.input_method());
                clipboard.merge(shell.clipboard_mut());

                if let Some(diff) = shell.is_layout_invalid() {
                    drop(maybe_overlay);

                    match diff {
                        shell::Diff::Perform => {
                            self.root.as_widget_mut().diff(&mut self.state);
                        }
                        shell::Diff::Skip => {}
                    }

                    self.base = self.root.as_widget_mut().layout(
                        &mut self.state,
                        renderer,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    maybe_overlay = self
                        .root
                        .as_widget_mut()
                        .overlay(
                            &mut self.state,
                            Layout::new(&self.base),
                            renderer,
                            &viewport,
                            Vector::ZERO,
                        )
                        .map(overlay::Nested::new);

                    if maybe_overlay.is_none() {
                        break;
                    }

                    overlay = maybe_overlay.as_mut().unwrap();

                    shell.revalidate_layout(|_diff| {
                        layout = overlay.layout(renderer, bounds);
                        has_layout_changed = true;
                    });
                }

                if shell.are_widgets_invalid() {
                    outdated = true;
                }
            }

            let (base_cursor, interaction) = if let Some(overlay) = maybe_overlay.as_mut() {
                let interaction = cursor
                    .position()
                    .map(|cursor_position| {
                        overlay.mouse_interaction(
                            Layout::new(&layout),
                            mouse::Cursor::Available(cursor_position),
                            renderer,
                        )
                    })
                    .unwrap_or_default();

                if interaction == mouse::Interaction::None {
                    (cursor, mouse::Interaction::None)
                } else {
                    (mouse::Cursor::Unavailable, interaction)
                }
            } else {
                (cursor, mouse::Interaction::None)
            };

            self.overlay = Some(Overlay {
                layout,
                interaction,
            });

            (base_cursor, event_statuses, interaction)
        } else {
            (
                cursor,
                vec![event::Status::Ignored; events.len()],
                mouse::Interaction::None,
            )
        };

        drop(maybe_overlay);

        let event_statuses = events
            .iter()
            .zip(overlay_statuses)
            .map(|(event, overlay_status)| {
                if matches!(overlay_status, event::Status::Captured) {
                    return overlay_status;
                }

                let mut shell = Shell::new(window, waker.clone(), messages);

                self.root.as_widget_mut().update(
                    &mut self.state,
                    event,
                    Layout::new(&self.base),
                    base_cursor,
                    renderer,
                    &mut shell,
                    &viewport,
                );

                if shell.event_status() == event::Status::Captured {
                    self.overlay = None;
                }

                redraw_request = redraw_request.min(shell.redraw_request());
                input_method.merge(shell.input_method());
                clipboard.merge(shell.clipboard_mut());

                shell.revalidate_layout(|diff| {
                    has_layout_changed = true;

                    match diff {
                        shell::Diff::Perform => {
                            self.root.as_widget_mut().diff(&mut self.state);
                        }
                        shell::Diff::Skip => {}
                    }

                    self.base = self.root.as_widget_mut().layout(
                        &mut self.state,
                        renderer,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    if let Some(mut overlay) = self
                        .root
                        .as_widget_mut()
                        .overlay(
                            &mut self.state,
                            Layout::new(&self.base),
                            renderer,
                            &viewport,
                            Vector::ZERO,
                        )
                        .map(overlay::Nested::new)
                    {
                        let layout = overlay.layout(renderer, self.bounds);
                        let interaction =
                            overlay.mouse_interaction(Layout::new(&layout), cursor, renderer);

                        self.overlay = Some(Overlay {
                            layout,
                            interaction,
                        });
                    }
                });

                if shell.are_widgets_invalid() {
                    outdated = true;
                }

                if shell.event_status() == event::Status::Ignored
                    && let Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::Tab),
                        modifiers,
                        ..
                    }) = event
                {
                    if modifiers.shift() {
                        let mut op = widget::operation::focusable::focus_previous::<()>();

                        self.operate(renderer, &mut op);
                    } else {
                        let mut op = widget::operation::focusable::focus_next::<()>();

                        self.operate(renderer, &mut op);
                    }

                    redraw_request = redraw_request.min(window::RedrawRequest::NextFrame);

                    return event::Status::Captured;
                }

                shell.event_status().merge(overlay_status)
            })
            .collect();

        let mouse_interaction = if overlay_interaction == mouse::Interaction::None {
            self.root.as_widget().mouse_interaction(
                &self.state,
                Layout::new(&self.base),
                base_cursor,
                &viewport,
                renderer,
            )
        } else {
            overlay_interaction
        };

        (
            if outdated {
                State::Outdated
            } else {
                State::Updated {
                    mouse_interaction,
                    redraw_request,
                    input_method,
                    clipboard,
                    has_layout_changed,
                }
            },
            event_statuses,
        )
    }

    /// Draws the [`UserInterface`] with the provided [`Renderer`].
    ///
    /// It returns the current [`mouse::Interaction`]. You should update the
    /// icon of the mouse cursor accordingly in your system.
    ///
    /// [`Renderer`]: crate::core::Renderer
    ///
    /// # Example
    /// We can finally draw our [counter](index.html#usage) by
    /// [completing the last example](#example-1):
    ///
    /// ```no_run
    /// # mod iced_wgpu {
    /// #     pub type Renderer = ();
    /// #     pub type Theme = ();
    /// # }
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> Element<(), (), Renderer> { unimplemented!() }
    /// #     pub fn update(&mut self, _: ()) {}
    /// # }
    /// use iced_runtime::core::mouse;
    /// use iced_runtime::core::renderer;
    /// use iced_runtime::core::shell;
    /// use iced_runtime::core::window;
    /// use iced_runtime::core::{Element, Size};
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::{Renderer, Theme};
    ///
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window = window::Headless; // This should be a proper window, like a `winit` one
    /// let mut waker = shell::Waker::noop();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor = mouse::Cursor::default();
    /// let mut events = Vec::new();
    /// let mut messages = shell::Bus::new();
    /// let mut theme = Theme::default();
    ///
    /// loop {
    ///     // Obtain system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     // Update the user interface
    ///     let event_statuses = user_interface.update(
    ///         &window,
    ///         &waker,
    ///         &events,
    ///         cursor,
    ///         &mut renderer,
    ///         &mut messages
    ///     );
    ///
    ///     // Draw the user interface
    ///     let mouse_interaction = user_interface.draw(&mut renderer, &theme, &renderer::Style::default(), cursor);
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     for message in messages.drain() {
    ///         counter.update(message);
    ///     }
    ///
    ///     // Update mouse cursor icon...
    ///     // Flush rendering operations...
    /// }
    /// ```
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        cursor: mouse::Cursor,
    ) {
        let viewport = Rectangle::with_size(self.bounds);
        renderer.reset(viewport);

        let base_cursor = match &self.overlay {
            None
            | Some(Overlay {
                interaction: mouse::Interaction::None,
                ..
            }) => cursor,
            _ => mouse::Cursor::Unavailable,
        };

        self.root.as_widget().draw(
            &self.state,
            renderer,
            theme,
            style,
            Layout::new(&self.base),
            base_cursor,
            &viewport,
        );

        let Self {
            overlay,
            root,
            base,
            ..
        } = self;

        let Some(Overlay { layout, .. }) = overlay.as_ref() else {
            return;
        };

        let overlay = root
            .as_widget_mut()
            .overlay(
                &mut self.state,
                Layout::new(base),
                renderer,
                &viewport,
                Vector::ZERO,
            )
            .map(overlay::Nested::new);

        if let Some(mut overlay) = overlay {
            overlay.draw(renderer, theme, style, Layout::new(layout), cursor);
        }
    }

    /// Applies a [`widget::Operation`] to the [`UserInterface`].
    pub fn operate(&mut self, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        self.operate_once(renderer, operation);

        let mut outcome = operation.finish();

        while let widget::operation::Outcome::Chain(mut next) = outcome {
            self.operate_once(renderer, &mut *next);
            outcome = next.finish();
        }
    }

    fn operate_once(&mut self, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        let viewport = Rectangle::with_size(self.bounds);

        self.root.as_widget_mut().operate(
            &mut self.state,
            Layout::new(&self.base),
            renderer,
            operation,
        );

        if let Some(mut overlay) = self
            .root
            .as_widget_mut()
            .overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
                &viewport,
                Vector::ZERO,
            )
            .map(overlay::Nested::new)
        {
            if self.overlay.is_none() {
                self.overlay = Some(Overlay {
                    layout: overlay.layout(renderer, self.bounds),
                    interaction: mouse::Interaction::None,
                });
            }

            overlay.operate(
                Layout::new(&self.overlay.as_ref().unwrap().layout),
                renderer,
                operation,
            );
        }
    }

    /// Returns the accessibility tree update for the [`UserInterface`].
    #[cfg(feature = "accessibility")]
    pub fn accessibility_tree(&mut self, renderer: &Renderer) -> accesskit::TreeUpdate {
        // Reset focus flags from the previous frame so only widgets that
        // are currently focused report themselves.
        reset_accesskit_focus(&self.state);

        let mut nodes = Vec::new();
        let mut id_counter = 1u64;

        let root_id = self.root.as_widget().accessibility(
            Layout::new(&self.base),
            &self.state,
            &mut nodes,
            &mut id_counter,
        );

        let root = root_id.unwrap_or(accesskit::NodeId(0));
        let focus = find_focused_node_id(&self.state).unwrap_or(root);

        // Build overlay accessibility tree
        let viewport = Rectangle::with_size(self.bounds);
        if let Some(mut overlay) = self
            .root
            .as_widget_mut()
            .overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
                &viewport,
                Vector::ZERO,
            )
            .map(crate::core::overlay::Nested::new)
        {
            let overlay_start = nodes.len();
            let overlay_root_id =
                overlay.accessibility(renderer, self.bounds, &mut nodes, &mut id_counter);

            // Attach overlay root as child of the widget that owns it
            // (e.g., an expanded ComboBox) rather than the window root.
            // This lets screen readers discover menu items as belonging to
            // the popup button they originated from.
            if let Some(overlay_root_id) = overlay_root_id {
                let overlay_nodes = &nodes[overlay_start..];
                let has_menu_popup = overlay_nodes
                    .iter()
                    .any(|(_, n)| n.role() == accesskit::Role::MenuListPopup);
                let has_tooltip = overlay_nodes
                    .iter()
                    .any(|(_, n)| n.role() == accesskit::Role::Tooltip);

                let popup_parent_id = if has_menu_popup {
                    nodes[..overlay_start]
                        .iter()
                        .rev()
                        .find(|(_, n)| {
                            n.role() == accesskit::Role::ComboBox && n.is_expanded() == Some(true)
                        })
                        .map(|(id, _)| *id)
                } else if has_tooltip {
                    nodes[..overlay_start]
                        .iter()
                        .rev()
                        .find(|(_, n)| {
                            n.has_popup() == Some(accesskit::HasPopup::Menu)
                                && n.is_expanded() == Some(true)
                        })
                        .map(|(id, _)| *id)
                } else {
                    None
                }
                .unwrap_or(root);

                if let Some((_, parent_node)) =
                    nodes.iter_mut().find(|(id, _)| *id == popup_parent_id)
                {
                    let mut children = parent_node.children().to_vec();

                    if !children.contains(&overlay_root_id) {
                        children.push(overlay_root_id);
                    }

                    parent_node.set_children(children);
                    parent_node.set_controls(&[overlay_root_id]);
                }
            }
        }

        accesskit::TreeUpdate {
            nodes,
            tree: Some(accesskit::Tree::new(root)),
            focus,
            tree_id: accesskit::TreeId::ROOT,
        }
    }

    /// Handles an accessibility action request by dispatching it to the
    /// appropriate widget in the tree, including any active overlays.
    #[cfg(feature = "accessibility")]
    pub fn handle_accessibility_action(
        &mut self,
        renderer: &Renderer,
        request: &accesskit::ActionRequest,
        shell: &mut Shell<'_, Message>,
    ) {
        if request.action == accesskit::Action::Focus {
            let mut operation = widget::operation::focusable::unfocus::<()>();

            self.operate(renderer, &mut operation);
        }

        // Dispatch to root widget tree
        if self.state.contains_accesskit_node_id(request.target_node) {
            self.root.as_widget_mut().accessibility_action(
                &mut self.state,
                Layout::new(&self.base),
                request,
                shell,
            );
        }

        // Also dispatch to active overlays
        let viewport = Rectangle::with_size(self.bounds);
        if let Some(mut overlay) = self
            .root
            .as_widget_mut()
            .overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
                &viewport,
                Vector::ZERO,
            )
            .map(overlay::Nested::new)
        {
            overlay.accessibility_action(renderer, self.bounds, request, shell);
        }
    }

    /// Revalidates the current widget layout using the given [`shell::Diff`].
    pub fn revalidate_layout(&mut self, renderer: &mut Renderer, diff: shell::Diff) {
        match diff {
            shell::Diff::Perform => {
                self.root.as_widget_mut().diff(&mut self.state);
            }
            shell::Diff::Skip => {}
        }

        self.base = self.root.as_widget_mut().layout(
            &mut self.state,
            renderer,
            &layout::Limits::new(Size::ZERO, self.bounds),
        );

        self.overlay = None;
    }

    /// Relayouts and returns a new  [`UserInterface`] using the provided
    /// bounds.
    pub fn relayout(self, bounds: Size, renderer: &mut Renderer) -> Self {
        Self::build(self.root, bounds, Cache { state: self.state }, renderer)
    }

    /// Extract the [`Cache`] of the [`UserInterface`], consuming it in the
    /// process.
    pub fn into_cache(self) -> Cache {
        Cache { state: self.state }
    }
}

/// Reusable data of a specific [`UserInterface`].
#[derive(Debug)]
pub struct Cache {
    state: widget::Tree,
}

impl Cache {
    /// Creates an empty [`Cache`].
    ///
    /// You should use this to initialize a [`Cache`] before building your first
    /// [`UserInterface`].
    pub fn new() -> Cache {
        Cache {
            state: widget::Tree::empty(),
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache::new()
    }
}

/// The resulting state after updating a [`UserInterface`].
#[derive(Debug)]
pub enum State {
    /// The [`UserInterface`] is outdated and needs to be rebuilt.
    Outdated,

    /// The [`UserInterface`] is up-to-date and can be reused without
    /// rebuilding.
    Updated {
        /// The current [`mouse::Interaction`] of the user interface.
        mouse_interaction: mouse::Interaction,
        /// The [`window::RedrawRequest`] describing when a redraw should be performed.
        redraw_request: window::RedrawRequest,
        /// The current [`InputMethod`] strategy of the user interface.
        input_method: InputMethod,
        /// The set of [`Clipboard`] requests that the user interface has produced.
        clipboard: Clipboard,
        /// Whether the layout of the [`UserInterface`] has changed.
        has_layout_changed: bool,
    },
}

impl State {
    /// Returns whether the layout of the [`UserInterface`] has changed.
    pub fn has_layout_changed(&self) -> bool {
        match self {
            State::Outdated => true,
            State::Updated {
                has_layout_changed, ..
            } => *has_layout_changed,
        }
    }
}

/// Recursively walks the widget [`Tree`] to find the first node that has
/// keyboard focus, returning its accesskit [`NodeId`].
#[cfg(feature = "accessibility")]
fn find_focused_node_id(tree: &widget::Tree) -> Option<accesskit::NodeId> {
    if tree.accesskit_focused() {
        return tree.accesskit_node_id();
    }

    for child in &tree.children {
        if let Some(id) = find_focused_node_id(child) {
            return Some(id);
        }
    }

    None
}

/// Resets the `accesskit_focused` flag on every node in the widget tree.
///
/// This must be called before each accessibility tree build so that
/// focus flags from the previous frame don't persist.
#[cfg(feature = "accessibility")]
fn reset_accesskit_focus(tree: &widget::Tree) {
    tree.set_accesskit_focused(false);

    for child in &tree.children {
        reset_accesskit_focus(child);
    }
}
