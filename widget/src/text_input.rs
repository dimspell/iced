//! Text inputs display fields that can be filled with text.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::text_input;
//!
//! struct State {
//!    content: String,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     ContentChanged(String)
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     text_input("Type something here...", &state.content)
//!         .on_input(Message::ContentChanged)
//!         .into()
//! }
//!
//! fn update(state: &mut State, message: Message) {
//!     match message {
//!         Message::ContentChanged(content) => {
//!             state.content = content;
//!         }
//!     }
//! }
//! ```
use crate::core::keyboard;
use crate::core::input_method;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::shell;
use crate::core::text;
use crate::core::text::editor;
use crate::core::text::input;
use crate::core::widget;
use crate::core::widget::operation::{self, Focusable, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    Background, Border, Color, Element, Event, Layout, Length, Padding, Pixels, Rectangle, Shell,
    Size, Theme, Widget,
};

#[cfg(feature = "accessibility")]
use std::cell::Cell;

/// A field that can be filled with text.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::text_input;
///
/// struct State {
///    content: String,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     ContentChanged(String)
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     text_input("Type something here...", &state.content)
///         .on_input(Message::ContentChanged)
///         .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::ContentChanged(content) => {
///             state.content = content;
///         }
///     }
/// }
/// ```
pub struct TextInput<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<widget::Id>,
    placeholder: text::Fragment<'a>,
    value: text::Fragment<'a>,
    is_secure: bool,
    is_read_only: bool,
    font: Option<Renderer::Font>,
    width: Length,
    height: Length,
    padding: Padding,
    size: Option<Pixels>,
    line_height: text::LineHeight,
    alignment: text::Alignment,
    multiline: Option<text::Wrapping>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    class: Theme::Class<'a>,
    last_status: Option<Status>,
    accessible_label: Option<String>,
    accessible_description: Option<String>,
    accessible_value: Option<String>,
}

/// The default [`Padding`] of a [`TextInput`].
pub const DEFAULT_PADDING: Padding = Padding::new(5.0);

impl<'a, Message, Theme, Renderer> TextInput<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new [`TextInput`] with the given placeholder and
    /// its current value.
    pub fn new(
        placeholder: impl text::IntoFragment<'a>,
        value: impl text::IntoFragment<'a>,
    ) -> Self {
        TextInput {
            id: None,
            placeholder: placeholder.into_fragment(),
            value: value.into_fragment(),
            is_secure: false,
            is_read_only: false,
            font: None,
            width: Length::Fill,
            height: Length::Fit,
            padding: DEFAULT_PADDING,
            size: None,
            line_height: text::LineHeight::default(),
            alignment: text::Alignment::Default,
            multiline: None,
            on_input: None,
            on_paste: None,
            on_submit: None,
            class: Theme::default(),
            last_status: None,
            accessible_label: None,
            accessible_description: None,
            accessible_value: None,
        }
    }

    /// Sets the [`widget::Id`] of the [`TextInput`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Converts the [`TextInput`] into a secure password input.
    pub fn secure(mut self, is_secure: bool) -> Self {
        self.is_secure = is_secure;
        self
    }

    /// Sets whether the [`TextInput`] is read-only.
    ///
    /// A read-only input can still be focused, navigated, selected, and copied,
    /// but its value cannot be edited.
    pub fn read_only(mut self, is_read_only: bool) -> Self {
        self.is_read_only = is_read_only;
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`].
    ///
    /// If this method is not called, the [`TextInput`] will be disabled.
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`], if `Some`.
    ///
    /// If `None`, the [`TextInput`] will be disabled.
    pub fn on_input_maybe(mut self, on_input: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_input = on_input.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed, if `Some`.
    pub fn on_submit_maybe(mut self, on_submit: Option<Message>) -> Self {
        self.on_submit = on_submit;
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`].
    pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`], if `Some`.
    pub fn on_paste_maybe(mut self, on_paste: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_paste = on_paste.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the [`Font`] of the [`TextInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`text::LineHeight`] of the [`TextInput`].
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the [`TextInput`].
    pub fn align_x(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.alignment = alignment.into();
        self
    }

    /// Sets the multiline behavior of the [`TextInput`].
    ///
    /// `None` will behave as a single line input.
    pub fn multiline(mut self, wrapping: Option<text::Wrapping>) -> Self {
        self.multiline = wrapping;
        self
    }

    /// Sets the style of the [`TextInput`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`TextInput`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the accessible label of the [`TextInput`].
    ///
    /// This is used by screen readers and other assistive technologies
    /// to describe the purpose of the text input.
    ///
    /// # Example
    /// ```ignore
    /// text_input("", "Search...", &query, Message::SearchChanged)
    ///     .accessible_label("Search query")
    /// ```
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Sets the accessible description of the [`TextInput`].
    pub fn accessible_description(mut self, description: impl Into<String>) -> Self {
        self.accessible_description = Some(description.into());
        self
    }

    /// Overrides the accessible value of the [`TextInput`].
    pub fn accessible_value(mut self, value: impl Into<String>) -> Self {
        self.accessible_value = Some(value.into());
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TextInput<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer>::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer>>();

        if state.value != self.value
            && state
                .transaction
                .as_ref()
                .is_none_or(shell::Tracking::is_processed)
        {
            state.input.overwrite(self.value.as_ref());
            state.value = self.value.clone().into_owned();
        }

        state.input.layout(
            renderer,
            limits,
            input::Layout {
                width: self.width,
                height: self.height,
                padding: self.padding,
                placeholder: self.placeholder.as_ref(),
                font: self.font,
                size: self.size,
                line_height: self.line_height,
                alignment: self.alignment,
                multiline: self.multiline,
                is_secure: self.is_secure,
            },
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer>>();

        operation.text_input(self.id.as_ref(), layout.bounds(), state);
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }

    #[cfg(feature = "accessibility")]
    fn accessibility(
        &self,
        layout: crate::core::Layout<'_>,
        tree: &crate::core::widget::Tree,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        id_counter: &mut u64,
    ) -> Option<accesskit::NodeId> {
        let id = accesskit::NodeId(*id_counter);
        tree.set_accesskit_node_id(id);
        *id_counter += 1;

        let role = if self.is_secure {
            accesskit::Role::PasswordInput
        } else {
            accesskit::Role::TextInput
        };
        let mut builder = accesskit::Node::new(role);
        crate::core::accessibility::set_bounds(tree, &mut builder, layout.bounds());

        crate::core::accessibility::apply_metadata(
            &mut builder,
            self.accessible_label.as_deref(),
            self.accessible_description.as_deref(),
            None,
        );

        // Use placeholder as name if value is empty
        let state = tree.state.downcast_ref::<State<Renderer>>();
        let committed_text = self.value.to_string();

        let cursor = state.input.cursor();
        let committed_char_lengths = char_byte_lengths(&committed_text);
        let cursor_char = byte_to_char_index(
            &committed_char_lengths,
            line_col_byte_offset(&committed_text, cursor.position.line, cursor.position.index),
        );
        let sel_char = cursor
            .selection
            .map(|s| {
                byte_to_char_index(
                    &committed_char_lengths,
                    line_col_byte_offset(&committed_text, s.line, s.index),
                )
            })
            .unwrap_or(cursor_char);

        let (mut text, sel_char, cursor_char) =
            text_with_preedit(committed_text, state.input.preedit(), sel_char, cursor_char);

        if self.is_secure {
            text = secure_text(&text);
        }

        let exposed_value = if self.is_secure {
            text.as_str()
        } else {
            self.accessible_value.as_deref().unwrap_or(&text)
        };

        if state
            .input
            .preedit()
            .is_some_and(|preedit| !preedit.content.is_empty())
        {
            builder.set_text_input_marked();
        }

        if exposed_value.is_empty() {
            if !self.placeholder.is_empty() {
                builder.set_placeholder(&*self.placeholder.clone().into_owned());
            }
        } else {
            builder.set_value(exposed_value);
        }

        // Create TextRun child for text range support, enabling
        // character-by-character and word-level feedback on macOS VoiceOver.
        let text_run_id = accesskit::NodeId(*id_counter);
        *id_counter += 1;
        state.accesskit_text_run_id.set(Some(text_run_id));

        let char_lengths: Vec<u8> = char_byte_lengths(&text);
        let word_starts: Vec<u8> = word_start_indices(&text);
        let mut text_run = accesskit::Node::new(accesskit::Role::TextRun);
        text_run.set_value(text.as_str());
        text_run.set_character_lengths(char_lengths.clone().into_boxed_slice());
        text_run.set_word_starts(word_starts.into_boxed_slice());

        builder.set_text_selection(accesskit::TextSelection {
            anchor: accesskit::TextPosition {
                node: text_run_id,
                character_index: sel_char,
            },
            focus: accesskit::TextPosition {
                node: text_run_id,
                character_index: cursor_char,
            },
        });

        builder.push_child(text_run_id);

        let is_disabled = self.on_input.is_none() && !self.is_read_only;
        let is_editable = self.on_input.is_some() && !self.is_read_only;

        if is_disabled {
            builder.set_disabled();
        } else {
            builder.add_action(accesskit::Action::SetTextSelection);
        }

        if self.is_read_only {
            builder.set_read_only();
        } else if is_editable {
            builder.add_action(accesskit::Action::ReplaceSelectedText);
            builder.add_action(accesskit::Action::SetValue);
        }

        builder.add_action(accesskit::Action::Focus);

        // Track keyboard focus for the accessibility tree
        if state.is_focused() {
            tree.set_accesskit_focused(true);
        }

        nodes.push((id, builder));
        nodes.push((text_run_id, text_run));

        Some(id)
    }

    #[cfg(feature = "accessibility")]
    fn accessibility_action(
        &mut self,
        tree: &mut crate::core::widget::Tree,
        _layout: crate::core::Layout<'_>,
        action: &accesskit::ActionRequest,
        shell: &mut crate::core::Shell<'_, Message>,
    ) {
        if !tree.owns_accesskit_node_id(action.target_node) {
            if action.action == accesskit::Action::Focus {
                tree.state
                    .downcast_mut::<State<Renderer>>()
                    .unfocus();
            }

            return;
        }

        if matches!(
            action.action,
            accesskit::Action::ReplaceSelectedText | accesskit::Action::SetValue
        ) && !self.is_read_only
        {
            if let Some(data) = &action.data {
                if let accesskit::ActionData::Value(value) = data {
                    if let Some(on_input) = &self.on_input {
                        shell.publish((on_input)(value.to_string()));
                        shell.request_redraw();
                    }
                }
            }
        } else if action.action == accesskit::Action::SetTextSelection
            && (self.on_input.is_some() || self.is_read_only)
        {
            let Some(accesskit::ActionData::SetTextSelection(selection)) = &action.data else {
                return;
            };
            let state = tree.state.downcast_mut::<State<Renderer>>();

            if state
                .input
                .preedit()
                .is_some_and(|preedit| !preedit.content.is_empty())
            {
                return;
            }

            let Some(text_run_id) = state.accesskit_text_run_id.get() else {
                return;
            };

            if selection.anchor.node != text_run_id || selection.focus.node != text_run_id {
                return;
            }

            let value = self.value.to_string();
            let length = char_byte_lengths(&value).len();
            let anchor = selection.anchor.character_index.min(length);
            let focus = selection.focus.character_index.min(length);

            operation::TextInput::select_range(
                &mut state.input,
                grapheme_index_to_position(&value, focus),
                grapheme_index_to_position(&value, anchor),
            );

            shell.request_redraw();
        } else if action.action == accesskit::Action::Focus {
            tree.state
                .downcast_mut::<State<Renderer>>()
                .focus();
            shell.request_redraw();
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = state::<Renderer>(tree);
        let is_disabled = self.on_input.is_none() && !self.is_read_only;

        if let Some(on_input) = &self.on_input {
let edit = if self.is_read_only {
                None
            } else {
                state
                    .input
                    .update(event, layout.bounds(), cursor, shell, |key_press| {
                        if let Some(on_submit) = &self.on_submit
                            && key_press.is_focused
                            && key_press.modified_key
                                == keyboard::Key::Named(keyboard::key::Named::Enter)
                        {
                            return Some(editor::Binding::Custom(on_submit.clone()));
                        }

                        editor::Binding::from_key_press(key_press)
                    })
            };

            if let Some(edit) = edit {
                let on_input = if let Some(on_paste) = &self.on_paste
                    && edit.has_pasted
                {
                    on_paste
                } else {
                    on_input
                };

                state.value = state.input.value();
                state.transaction = Some(shell.publish_and_track(on_input(state.value.clone())));
            }
        }

        let status = if is_disabled {
            Status::Disabled
        } else if state.input.is_focused() {
            Status::Focused {
                is_hovered: cursor.is_over(layout.bounds()),
            }
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);

            shell.request_input_method(
                &state
                    .input
                    .input_method(layout.bounds().shrink(self.padding).position()),
            );
        } else if self
            .last_status
            .is_some_and(|last_status| status != last_status)
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer>>();
        let style = theme.style(&self.class, self.last_status.unwrap_or(Status::Disabled));
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        state.input.draw(
            renderer,
            bounds,
            *viewport,
            input::Style {
                value: style.value,
                selection: style.selection,
                placeholder: style.placeholder,
            },
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            if self.on_input.is_none() && !self.is_read_only {
                mouse::Interaction::Idle
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<TextInput<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'static,
{
    fn from(
        text_input: TextInput<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text_input)
    }
}

/// The state of a [`TextInput`].
struct State<R: text::Renderer> {
    input: text::Input<R>,
    value: String,
    transaction: Option<shell::Tracking>,
    #[cfg(feature = "accessibility")]
    accesskit_text_run_id: Cell<Option<accesskit::NodeId>>,
}

fn state<Renderer: text::Renderer + 'static>(tree: &mut Tree) -> &mut State<Renderer> {
    tree.state.downcast_mut::<State<Renderer>>()
}

impl<R: text::Renderer> State<R> {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    fn new() -> Self {
        Self {
            input: text::Input::new(),
            value: String::new(),
            transaction: None,
            #[cfg(feature = "accessibility")]
            accesskit_text_run_id: Cell::new(None),
        }
    }
}

impl<R: text::Renderer> operation::Focusable for State<R> {
    fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    fn focus(&mut self) {
        self.input.focus();
    }

    fn unfocus(&mut self) {
        self.input.unfocus();
    }
}

impl<R: text::Renderer> operation::TextInput for State<R> {
    fn text(&self) -> text::Fragment<'_> {
        if self.input.is_empty() {
            text::Fragment::Borrowed(self.input.placeholder())
        } else {
            text::Fragment::Owned(self.input.value())
        }
    }

    fn move_cursor_to_front(&mut self) {
        self.input.move_cursor_to_front();
    }

    fn move_cursor_to_end(&mut self) {
        self.input.move_cursor_to_end();
    }

    fn move_cursor_to(&mut self, position: text::Position) {
        self.input.move_cursor_to(position);
    }

    fn select_all(&mut self) {
        self.input.select_all();
    }

    fn select_range(&mut self, start: text::Position, end: text::Position) {
        self.input.select_range(start, end);
    }
}

/// The possible status of a [`TextInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`TextInput`] can be interacted with.
    Active,
    /// The [`TextInput`] is being hovered.
    Hovered,
    /// The [`TextInput`] is focused.
    Focused {
        /// Whether the [`TextInput`] is hovered, while focused.
        is_hovered: bool,
    },
    /// The [`TextInput`] cannot be interacted with.
    Disabled,
}

/// The appearance of a text input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the text input.
    pub background: Background,
    /// The [`Border`] of the text input.
    pub border: Border,
    /// The [`Color`] of the placeholder of the text input.
    pub placeholder: Color,
    /// The [`Color`] of the value of the text input.
    pub value: Color,
    /// The [`Color`] of the selection of the text input.
    pub selection: Color,
}

/// The theme catalog of a [`TextInput`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`TextInput`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`TextInput`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let active = Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        placeholder: palette.secondary.base.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: Border {
                color: palette.background.base.text,
                ..active.border
            },
            ..active
        },
        Status::Focused { .. } => Style {
            border: Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
        Status::Disabled => Style {
            background: Background::Color(palette.background.weak.color),
            value: active.placeholder,
            placeholder: palette.background.strongest.color,
            ..active
        },
    }
}

#[cfg(feature = "accessibility")]
fn char_byte_lengths(text: &str) -> Vec<u8> {
    unicode_segmentation::UnicodeSegmentation::graphemes(text, true)
        .map(|g| g.len() as u8)
        .collect()
}

#[cfg(feature = "accessibility")]
fn word_start_indices(text: &str) -> Vec<u8> {
    let graphemes: Vec<&str> =
        unicode_segmentation::UnicodeSegmentation::graphemes(text, true).collect();
    let mut starts = Vec::new();
    let mut in_word = false;

    for (i, grapheme) in graphemes.iter().enumerate() {
        let is_word_char = grapheme.chars().any(|c| c.is_alphanumeric());
        if is_word_char && !in_word {
            starts.push(i as u8);
        }
        in_word = is_word_char;
    }

    starts
}

#[cfg(feature = "accessibility")]
fn line_col_byte_offset(text: &str, line: usize, index: usize) -> usize {
    let mut offset = 0;
    for (i, line_text) in text.lines().enumerate() {
        if i == line {
            offset += index;
            break;
        }
        offset += line_text.len() + 1; // +1 for newline character
    }
    offset.min(text.len())
}

#[cfg(feature = "accessibility")]
fn secure_text(text: &str) -> String {
    unicode_segmentation::UnicodeSegmentation::graphemes(text, true)
        .map(|_| "•")
        .collect()
}

#[cfg(feature = "accessibility")]
fn grapheme_index_to_position(text: &str, grapheme_index: usize) -> text::Position {
    use unicode_segmentation::UnicodeSegmentation;

    let byte_offset: usize = UnicodeSegmentation::graphemes(text, true)
        .take(grapheme_index)
        .map(str::len)
        .sum();
    let bytes = text.as_bytes();
    let mut line = 0;
    let mut line_start = 0;
    let mut index = 0;

    while index < bytes.len() {
        if matches!(bytes[index], b'\r' | b'\n') {
            if byte_offset <= index {
                return text::Position {
                    line,
                    index: byte_offset
                        .saturating_sub(line_start)
                        .min(index - line_start),
                };
            }

            let first = bytes[index];
            index += 1;

            if index < bytes.len()
                && matches!((first, bytes[index]), (b'\r', b'\n') | (b'\n', b'\r'))
            {
                index += 1;
            }

            if byte_offset < index {
                return text::Position {
                    line,
                    index: index - line_start,
                };
            }

            line += 1;
            line_start = index;
        } else {
            index += 1;
        }
    }

    text::Position {
        line,
        index: byte_offset
            .saturating_sub(line_start)
            .min(text.len() - line_start),
    }
}

#[cfg(feature = "accessibility")]
fn text_with_preedit(
    mut text: String,
    preedit: Option<&input_method::Preedit>,
    anchor: usize,
    focus: usize,
) -> (String, usize, usize) {
    let Some(preedit) = preedit.filter(|preedit| !preedit.content.is_empty()) else {
        return (text, anchor, focus);
    };

    let insertion = anchor.min(focus);
    let insertion_byte = {
        let char_lengths = char_byte_lengths(&text);

        char_lengths
            .iter()
            .take(insertion.min(char_lengths.len()))
            .map(|&len| len as usize)
            .sum()
    };
    text.insert_str(insertion_byte, &preedit.content);

    let selection = preedit.selection.as_ref().map_or_else(
        || {
            let len = char_byte_lengths(&preedit.content).len();
            len..len
        },
        |selection| {
            let char_lengths = char_byte_lengths(&preedit.content);

            byte_to_char_index(&char_lengths, selection.start)
                ..byte_to_char_index(&char_lengths, selection.end)
        },
    );

    (text, insertion + selection.start, insertion + selection.end)
}

#[cfg(feature = "accessibility")]
fn byte_to_char_index(char_lengths: &[u8], byte_offset: usize) -> usize {
    let mut accumulated = 0;

    for (i, &len) in char_lengths.iter().enumerate() {
        if accumulated >= byte_offset {
            return i;
        }

        accumulated += len as usize;
    }

    char_lengths.len()
}
