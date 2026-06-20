//! Integration tests for accessibility tree building.
use iced_test::simulator;
use iced_test::core::Theme;
use iced_test::renderer::Renderer;
use iced_widget::{
    button, checkbox, column, container, pick_list, progress_bar, radio, row, slider, text,
    text_input, toggler, vertical_slider,
};
use accesskit::Role;
use iced_test::core::Point; // for menu overlay positioning tests

/// Find a node in the tree with the given role.
fn find_node<'a>(
    tree: &'a accesskit::TreeUpdate,
    role: Role,
) -> Option<&'a accesskit::Node> {
    tree.nodes
        .iter()
        .find(|(_, n)| n.role() == role)
        .map(|(_, n)| n)
}

// === ROLES ===

#[test]
fn button_role() {
    let mut ui = simulator::<(), Theme, Renderer>(button("Click"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Button).expect("Button node");
    assert!(node.is_disabled(), "Button without on_press should be disabled");
    assert!(
        !node.supports_action(accesskit::Action::Click),
        "Disabled button should not advertise Click action"
    );
}

#[test]
fn button_with_on_press() {
    let mut ui = simulator::<(), Theme, Renderer>(button("Activate").on_press(()));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Button).expect("Button node");
    assert!(!node.is_disabled(), "Button with on_press should be enabled");
    assert!(
        node.supports_action(accesskit::Action::Click),
        "Enabled button should advertise Click action"
    );
}

#[test]
fn checkbox_role() {
    let mut ui = simulator::<(), Theme, Renderer>(checkbox(true).label("Accept").on_toggle(|_| {}));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::CheckBox).expect("CheckBox node");
    assert_eq!(node.label(), Some("Accept"));
}

#[test]
fn toggler_role() {
    let mut ui = simulator::<(), Theme, Renderer>(toggler(true).label("WiFi"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Switch).expect("Switch node");
    assert_eq!(node.label(), Some("WiFi"));
}

#[test]
fn radio_role() {
    let mut ui = simulator::<(), Theme, Renderer>(radio("Option A", 1, Some(1), |_| ()));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::RadioButton).expect("RadioButton node");
    assert_eq!(node.label(), Some("Option A"));
    assert!(
        node.supports_action(accesskit::Action::Click),
        "Radio should advertise Click action"
    );
}

#[test]
fn slider_numeric_range() {
    let mut ui = simulator::<(), Theme, Renderer>(
        slider(0..=100, 50, |_| {}).accessible_label("Volume"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Slider).expect("Slider node");
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(100.0));
    assert_eq!(node.numeric_value(), Some(50.0));
    assert_eq!(node.label(), Some("Volume"));
}

#[test]
fn vertical_slider_numeric_range() {
    let mut ui = simulator::<(), Theme, Renderer>(
        vertical_slider(0.0..=100.0, 50.0, |_| {}).accessible_label("Volume"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Slider).expect("Slider node");
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(100.0));
    assert_eq!(node.numeric_value(), Some(50.0));
    assert_eq!(node.label(), Some("Volume"));
}

#[test]
fn vertical_slider_actions() {
    let mut ui = simulator::<(), Theme, Renderer>(
        vertical_slider(0..=100, 50, |_| {}),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Slider).expect("Slider node");
    assert!(node.supports_action(accesskit::Action::Increment));
    assert!(node.supports_action(accesskit::Action::Decrement));
}

#[test]
fn text_input_value_and_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        text_input("", "Hello").accessible_label("Name"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::TextInput).expect("TextInput node");
    assert_eq!(node.value(), Some("Hello"));
    assert!(node.label().is_some());
}

#[test]
fn text_editor_label() {
    use iced_widget::text_editor;
    let content = text_editor::Content::new();
    let mut ui = simulator::<(), Theme, Renderer>(
        text_editor(&content).accessible_label("Description"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::TextInput).expect("TextInput node (TextEditor)");
    assert_eq!(node.label(), Some("Description"));
}

#[test]
fn progress_bar_properties() {
    let mut ui = simulator::<(), Theme, Renderer>(progress_bar(0.0..=100.0, 50.0));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::ProgressIndicator).expect("ProgressIndicator node");
    assert_eq!(node.numeric_value(), Some(0.5));
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(1.0));
    assert_eq!(node.live(), Some(accesskit::Live::Polite));
}

// === HEADINGS ===

#[test]
fn text_heading_level_1() {
    let mut ui = simulator::<(), Theme, Renderer>(text("Welcome").heading(1));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Heading).expect("Heading node");
    assert_eq!(node.level(), Some(1));
    assert_eq!(node.value(), Some("Welcome"));
}

#[test]
fn text_heading_level_3() {
    let mut ui = simulator::<(), Theme, Renderer>(text("Subtitle").heading(3));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Heading).expect("Heading node");
    assert_eq!(node.level(), Some(3));
}

// === INVISIBLE WIDGETS ===

#[test]
fn empty_text_no_node() {
    let mut ui = simulator::<(), Theme, Renderer>(text(""));
    let tree = ui.accessibility_tree();
    assert!(
        tree.nodes.is_empty(),
        "Empty text should not produce any nodes"
    );
}

#[test]
fn space_no_node() {
    let mut ui = simulator::<(), Theme, Renderer>(iced_widget::space::Space::new());
    let tree = ui.accessibility_tree();
    assert!(
        tree.nodes.is_empty(),
        "Space should not produce any nodes"
    );
}

// === COMPOSITIONS ===

#[test]
fn column_contains_children() {
    let mut ui = simulator::<(), Theme, Renderer>(
        column![button("A").on_press(()), button("B").on_press(())],
    );
    let tree = ui.accessibility_tree();
    // Should have: Group (column) + Button A + Label "A" + Button B + Label "B" = 5 nodes
    assert_eq!(tree.nodes.len(), 5, "Column + 2 buttons = 5 nodes");
    assert!(find_node(&tree, Role::Group).is_some());
    assert!(find_node(&tree, Role::Button).is_some());
}

#[test]
fn row_contains_children() {
    let mut ui = simulator::<(), Theme, Renderer>(
        row![button("A").on_press(()), button("B").on_press(())],
    );
    let tree = ui.accessibility_tree();
    assert_eq!(tree.nodes.len(), 5, "Row + 2 buttons = 5 nodes");
}

#[test]
fn container_wraps_child() {
    let mut ui = simulator::<(), Theme, Renderer>(
        container(text("Content")).width(100).height(50),
    );
    let tree = ui.accessibility_tree();
    // Should have: Group (container) + Label (text) = 2 nodes
    assert_eq!(tree.nodes.len(), 2);
    assert!(find_node(&tree, Role::Group).is_some());
    assert!(find_node(&tree, Role::Label).is_some());
}

// === TREE STRUCTURE ===

#[test]
fn tree_has_root_and_focus() {
    let mut ui = simulator::<(), Theme, Renderer>(button("Click"));
    let tree = ui.accessibility_tree();
    assert!(tree.tree.is_some(), "Tree should have a tree descriptor");
    let tree_info = tree.tree.as_ref().unwrap();
    let root_id = tree_info.root;
    assert!(tree.nodes.iter().any(|(id, _)| *id == root_id));
    // focus should always be set (defaults to root)
    assert!(tree.nodes.iter().any(|(id, _)| *id == tree.focus));
}

// === ACTION DISPATCH ===

#[test]
fn click_action_dispatches_button_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(button("Press me").on_press(42));
    let request = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: accesskit::NodeId(0),
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![42], "Button click should produce the on_press message");
}

#[test]
fn increment_action_dispatches_slider_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(slider(0..=100, 50, |v| v));
    // First get the tree to find the slider's node ID
    let tree = ui.accessibility_tree();
    let (slider_id, _) = tree.nodes.iter().find(|(_, n)| n.role() == Role::Slider).expect("Slider node");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Increment,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *slider_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![51], "Slider increment with step=1 should produce 51");
}

#[test]
fn decrement_action_dispatches_slider_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(slider(0..=100, 50, |v| v));
    let tree = ui.accessibility_tree();
    let (slider_id, _) = tree.nodes.iter().find(|(_, n)| n.role() == Role::Slider).expect("Slider node");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Decrement,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *slider_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![49], "Slider decrement with step=1 should produce 49");
}

// === MENU / DROPDOWN ACCESSIBILITY ===

#[test]
fn picklist_menu_items_visible_when_open() {
    let options = vec!["Option A", "Option B", "Option C"];
    let mut ui = simulator::<(), Theme, Renderer>(
        pick_list(Some("Option A"), options.as_slice(), |s: &&str| s.to_string())
            .on_select(|_| ()),
    );

    // Click to open the dropdown — PickList should start at origin
    ui.point_at(Point::new(10.0, 10.0));
    let _ = ui.simulate(iced_test::simulator::click());

    let tree = ui.accessibility_tree();
    let items: Vec<&accesskit::Node> = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::MenuItem)
        .map(|(_, n)| n)
        .collect();

    assert_eq!(items.len(), 3, "Should have 3 MenuItem nodes when dropdown is open");
    assert_eq!(items[0].label(), Some("Option A"));
    assert!(items[0].supports_action(accesskit::Action::Click));
}

#[test]
fn picklist_menu_item_click_dispatches_selection() {
    let options = vec!["Alpha", "Beta", "Gamma"];
    let mut ui = simulator::<String, Theme, Renderer>(
        pick_list(
            Some("Alpha"),
            options.as_slice(),
            |s: &&str| s.to_string(),
        )
        .on_select(|s| s.to_string()),
    );

    // Open the dropdown
    ui.point_at(Point::new(10.0, 10.0));
    let _ = ui.simulate(iced_test::simulator::click());

    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| {
            n.role() == Role::MenuItem
                && n.label() == Some("Beta")
        })
        .expect("Second menu item 'Beta' should exist");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *target_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<String> = ui.into_messages().collect();
    assert_eq!(messages, vec!["Beta"], "Clicking menu item should produce its selected message");
}
