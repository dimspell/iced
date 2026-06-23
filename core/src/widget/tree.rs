//! Store internal widget state in a state tree to ensure continuity.
use crate::Widget;

use std::any::{self, Any};
use std::borrow::{Borrow, BorrowMut};
#[cfg(feature = "accessibility")]
use std::cell::Cell;
use std::fmt;

/// A persistent state widget tree.
///
/// A [`Tree`] is normally associated with a specific widget in the widget tree.
#[derive(Debug)]
pub struct Tree {
    /// The tag of the [`Tree`].
    pub tag: Tag,

    /// The [`State`] of the [`Tree`].
    pub state: State,

    /// The children of the root widget of the [`Tree`].
    pub children: Vec<Tree>,

    /// The accesskit [`NodeId`] assigned to this widget during the last
    /// accessibility tree build, if any.
    #[cfg(feature = "accessibility")]
    pub accesskit_node_id: Cell<Option<accesskit::NodeId>>,

    /// Whether this widget currently has keyboard focus, determined during the
    /// last accessibility tree build.
    #[cfg(feature = "accessibility")]
    pub accesskit_focused: Cell<bool>,
}

impl Tree {
    /// Creates an empty, stateless [`Tree`] with no children.
    pub fn empty() -> Self {
        Self {
            tag: Tag::stateless(),
            state: State::None,
            children: Vec::new(),
            #[cfg(feature = "accessibility")]
            accesskit_node_id: Cell::new(None),
            #[cfg(feature = "accessibility")]
            accesskit_focused: Cell::new(false),
        }
    }

    /// Creates a new [`Tree`] for the provided [`Widget`].
    pub fn new<'a, Message, Theme, Renderer>(
        widget: impl Borrow<dyn Widget<Message, Theme, Renderer> + 'a>,
    ) -> Self
    where
        Renderer: crate::Renderer,
    {
        let widget = widget.borrow();

        Self {
            tag: widget.tag(),
            state: widget.state(),
            children: Vec::new(),
            #[cfg(feature = "accessibility")]
            accesskit_node_id: Cell::new(None),
            #[cfg(feature = "accessibility")]
            accesskit_focused: Cell::new(false),
        }
    }

    /// Sets the accesskit [`NodeId`] assigned to this widget.
    #[cfg(feature = "accessibility")]
    pub fn set_accesskit_node_id(&self, id: accesskit::NodeId) {
        self.accesskit_node_id.set(Some(id));
    }

    /// Returns the accesskit [`NodeId`] assigned to this widget, if any.
    #[cfg(feature = "accessibility")]
    pub fn accesskit_node_id(&self) -> Option<accesskit::NodeId> {
        self.accesskit_node_id.get()
    }

    /// Returns whether this tree, or any of its children, contains the
    /// provided accesskit [`NodeId`].
    #[cfg(feature = "accessibility")]
    pub fn contains_accesskit_node_id(&self, id: accesskit::NodeId) -> bool {
        self.accesskit_node_id() == Some(id)
            || self
                .children
                .iter()
                .any(|child| child.contains_accesskit_node_id(id))
    }

    /// Returns whether this tree itself, or any of its descendants, owns the
    /// provided accesskit [`NodeId`].
    #[cfg(feature = "accessibility")]
    pub fn owns_accesskit_node_id(&self, id: accesskit::NodeId) -> bool {
        self.contains_accesskit_node_id(id)
    }

    /// Sets whether this widget has keyboard focus.
    #[cfg(feature = "accessibility")]
    pub fn set_accesskit_focused(&self, focused: bool) {
        self.accesskit_focused.set(focused);
    }

    /// Returns whether this widget has keyboard focus.
    #[cfg(feature = "accessibility")]
    pub fn accesskit_focused(&self) -> bool {
        self.accesskit_focused.get()
    }

    /// Reconciles the current tree with the provided [`Widget`].
    ///
    /// If the tag of the [`Widget`] matches the tag of the [`Tree`], then the
    /// [`Widget`] proceeds with the reconciliation (i.e. [`Widget::diff`] is called).
    ///
    /// Otherwise, the whole [`Tree`] is recreated.
    ///
    /// [`Widget::diff`]: crate::Widget::diff
    pub fn diff<'a, Message, Theme, Renderer>(
        &mut self,
        mut new: impl BorrowMut<dyn Widget<Message, Theme, Renderer> + 'a>,
    ) where
        Renderer: crate::Renderer,
    {
        if self.tag != new.borrow().tag() {
            *self = Self::new(new.borrow());
        }

        new.borrow_mut().diff(self);
    }

    /// Reconciles the children of the tree with the provided list of widgets.
    pub fn diff_children<'a, Message, Theme, Renderer>(
        &mut self,
        new_children: &mut [impl BorrowMut<dyn Widget<Message, Theme, Renderer> + 'a>],
    ) where
        Renderer: crate::Renderer,
    {
        self.diff_children_custom(
            new_children,
            |tree, widget| tree.diff(widget.borrow_mut()),
            |widget| Self::new(widget.borrow()),
        );
    }

    /// Reconciles the children of the tree with the provided list of widgets using custom
    /// logic both for diffing and creating new widget state.
    pub fn diff_children_custom<T>(
        &mut self,
        new_children: &mut [T],
        diff: impl Fn(&mut Tree, &mut T),
        new_state: impl Fn(&T) -> Self,
    ) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        if self.children.len() < new_children.len() {
            self.children
                .extend(new_children[self.children.len()..].iter().map(new_state));
        }

        for (child_state, new) in self.children.iter_mut().zip(new_children.iter_mut()) {
            diff(child_state, new);
        }
    }
}

/// Reconciles the `current_children` with the provided list of widgets using
/// custom logic both for diffing and creating new widget state.
///
/// The algorithm will try to minimize the impact of diffing by querying the
/// `maybe_changed` closure.
pub fn diff_children_custom_with_search<T>(
    current_children: &mut Vec<Tree>,
    new_children: &mut [T],
    diff: impl Fn(&mut Tree, &mut T),
    maybe_changed: impl Fn(usize) -> bool,
    new_state: impl Fn(&T) -> Tree,
) {
    if new_children.is_empty() {
        current_children.clear();
        return;
    }

    if current_children.is_empty() {
        current_children.extend(new_children.iter().map(new_state));

        // TODO: Merge loop with extend logic (?)
        for (child_state, new) in current_children.iter_mut().zip(new_children.iter_mut()) {
            diff(child_state, new);
        }

        return;
    }

    let first_maybe_changed = maybe_changed(0);
    let last_maybe_changed = maybe_changed(current_children.len() - 1);

    if current_children.len() > new_children.len() {
        if !first_maybe_changed && last_maybe_changed {
            current_children.truncate(new_children.len());
        } else {
            let difference_index = if first_maybe_changed {
                0
            } else {
                (1..current_children.len())
                    .find(|&i| maybe_changed(i))
                    .unwrap_or(0)
            };

            let _ = current_children.splice(
                difference_index..difference_index + (current_children.len() - new_children.len()),
                std::iter::empty(),
            );
        }
    }

    if current_children.len() < new_children.len() {
        let first_maybe_changed = maybe_changed(0);
        let last_maybe_changed = maybe_changed(current_children.len() - 1);

        if !first_maybe_changed && last_maybe_changed {
            current_children.extend(new_children[current_children.len()..].iter().map(new_state));
        } else {
            let difference_index = if first_maybe_changed {
                0
            } else {
                (1..current_children.len())
                    .find(|&i| maybe_changed(i))
                    .unwrap_or(0)
            };

            let _ = current_children.splice(
                difference_index..difference_index,
                new_children[difference_index
                    ..difference_index + (new_children.len() - current_children.len())]
                    .iter()
                    .map(new_state),
            );
        }
    }

    // TODO: Merge loop with extend logic (?)
    for (child_state, new) in current_children.iter_mut().zip(new_children.iter_mut()) {
        diff(child_state, new);
    }
}

/// The identifier of some widget state.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Tag(any::TypeId);

impl Tag {
    /// Creates a [`Tag`] for a state of type `T`.
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        Self(any::TypeId::of::<T>())
    }

    /// Creates a [`Tag`] for a stateless widget.
    pub fn stateless() -> Self {
        Self::of::<()>()
    }
}

/// The internal [`State`] of a widget.
pub enum State {
    /// No meaningful internal state.
    None,

    /// Some meaningful internal state.
    Some(Box<dyn Any>),
}

impl State {
    /// Creates a new [`State`].
    pub fn new<T>(state: T) -> Self
    where
        T: 'static,
    {
        State::Some(Box::new(state))
    }

    /// Downcasts the [`State`] to `T` and returns a reference to it.
    ///
    /// # Panics
    /// This method will panic if the downcast fails or the [`State`] is [`State::None`].
    pub fn downcast_ref<T>(&self) -> &T
    where
        T: 'static,
    {
        match self {
            State::None => panic!("Downcast on stateless state"),
            State::Some(state) => state.downcast_ref().expect("Downcast widget state"),
        }
    }

    /// Downcasts the [`State`] to `T` and returns a mutable reference to it.
    ///
    /// # Panics
    /// This method will panic if the downcast fails or the [`State`] is [`State::None`].
    pub fn downcast_mut<T>(&mut self) -> &mut T
    where
        T: 'static,
    {
        match self {
            State::None => panic!("Downcast on stateless state"),
            State::Some(state) => state.downcast_mut().expect("Downcast widget state"),
        }
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "State::None"),
            Self::Some(_) => write!(f, "State::Some"),
        }
    }
}
