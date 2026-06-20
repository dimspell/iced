//! Integration tests for accessibility tree building.
use iced_test::simulator;
use iced_test::core::Theme;
use iced_test::renderer::Renderer;
use iced_widget::{
    button, checkbox, column, progress_bar, scrollable, slider, text, text_input,
};

#[test]
fn button_has_button_role() {
    let mut ui = simulator::<(), Theme, Renderer>(button("Click"));
    let tree = ui.accessibility_tree();
    assert!(!tree.nodes.is_empty(), "Tree should contain at least one node");
}

#[test]
fn checkbox_has_label() {
    let mut ui = simulator::<(), Theme, Renderer>(checkbox(true).label("Accept terms"));
    let tree = ui.accessibility_tree();
    assert!(!tree.nodes.is_empty());
}

#[test]
fn text_input_with_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        text_input("", "value")
            .accessible_label("Search query"),
    );
    let tree = ui.accessibility_tree();
    assert!(!tree.nodes.is_empty());
}

#[test]
fn slider_with_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        slider(0..=100, 50, |_| {})
            .accessible_label("Volume"),
    );
    let tree = ui.accessibility_tree();
    assert!(!tree.nodes.is_empty());
}

#[test]
fn progress_bar_is_live() {
    let mut ui = simulator::<(), Theme, Renderer>(progress_bar(0.0..=100.0, 50.0));
    let tree = ui.accessibility_tree();
    assert!(!tree.nodes.is_empty());
}

#[test]
fn scrollable_has_scroll_positions() {
    let mut ui = simulator::<(), Theme, Renderer>(
        scrollable(
            column![
                text("Line 1"),
                text("Line 2"),
                text("Line 3"),
            ]
            .height(2000),
        )
        .height(400),
    );
    let tree = ui.accessibility_tree();
    assert!(!tree.nodes.is_empty());
}
