//! Integration tests for accessibility tree building.
use accesskit::{Action, ActionRequest, Role, TreeId};
use iced_test::core::Point;
use iced_test::core::{Event, Theme, keyboard};
use iced_test::renderer::Renderer;
use iced_test::simulator;
use iced_widget::{
    button, checkbox, column, container, pick_list, progress_bar, radio, row, scrollable, slider,
    text, text_input, toggler, tooltip, vertical_slider,
}; // for menu overlay positioning tests

/// Find a node in the tree with the given role.
fn find_node<'a>(tree: &'a accesskit::TreeUpdate, role: Role) -> Option<&'a accesskit::Node> {
    tree.nodes
        .iter()
        .find(|(_, n)| n.role() == role)
        .map(|(_, n)| n)
}

fn has_non_empty_bounds(node: &accesskit::Node) -> bool {
    node.bounds()
        .is_some_and(|bounds| bounds.x1 > bounds.x0 && bounds.y1 > bounds.y0)
}

// === ROLES ===

#[test]
fn button_role() {
    let mut ui = simulator::<(), Theme, Renderer>(button("Click"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Button).expect("Button node");
    assert!(
        node.is_disabled(),
        "Button without on_press should be disabled"
    );
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
    assert!(
        !node.is_disabled(),
        "Button with on_press should be enabled"
    );
    assert!(
        node.supports_action(accesskit::Action::Click),
        "Enabled button should advertise Click action"
    );
}

#[test]
fn button_accessible_metadata_overrides_child_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        button("Visible")
            .accessible_label("Run")
            .accessible_description("Starts the job")
            .accessible_value("Ready")
            .on_press(()),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Button).expect("Button node");
    assert_eq!(node.label(), Some("Run"));
    assert_eq!(node.description(), Some("Starts the job"));
    assert_eq!(node.value(), Some("Ready"));
}

#[test]
fn focusable_controls_have_non_empty_bounds() {
    let mut ui = simulator::<(), Theme, Renderer>(column![
        button("Button").on_press(()),
        checkbox(false).label("Checkbox").on_toggle(|_| ()),
        toggler(false).label("Toggle").on_toggle(|_| ()),
        radio("Radio", 1, Some(0), |_| ()),
        slider(0..=100, 50, |_| ()).accessible_label("Slider"),
        vertical_slider(0..=100, 50, |_| ()).accessible_label("Vertical slider"),
        text_input("Placeholder", "").on_input(|_| ()),
        pick_list(Some("A"), ["A", "B"].as_slice(), |s: &&str| {
            s.to_string()
        })
        .on_select(|_| ()),
    ]);

    let tree = ui.accessibility_tree();

    for role in [
        Role::Button,
        Role::CheckBox,
        Role::Switch,
        Role::RadioButton,
        Role::Slider,
        Role::TextInput,
        Role::ComboBox,
    ] {
        let nodes: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == role)
            .collect();

        assert!(!nodes.is_empty(), "Expected at least one {role:?} node");

        for (_, node) in nodes {
            assert!(
                has_non_empty_bounds(node),
                "{role:?} node should have non-empty bounds"
            );
        }
    }
}

#[test]
fn checkbox_role() {
    let mut ui = simulator::<(), Theme, Renderer>(checkbox(true).label("Accept").on_toggle(|_| {}));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::CheckBox).expect("CheckBox node");
    assert_eq!(node.label(), Some("Accept"));
}

#[test]
fn checkbox_accessible_metadata() {
    let mut ui = simulator::<(), Theme, Renderer>(
        checkbox(true)
            .label("Visible")
            .accessible_label("Accept terms")
            .accessible_description("Required before continuing")
            .accessible_value("Accepted")
            .on_toggle(|_| {}),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::CheckBox).expect("CheckBox node");
    assert_eq!(node.label(), Some("Accept terms"));
    assert_eq!(node.description(), Some("Required before continuing"));
    assert_eq!(node.value(), Some("Accepted"));
}

#[test]
fn toggler_role() {
    let mut ui = simulator::<(), Theme, Renderer>(toggler(true).label("WiFi"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Switch).expect("Switch node");
    assert_eq!(node.label(), Some("WiFi"));
}

#[test]
fn toggler_accessible_metadata() {
    let mut ui = simulator::<(), Theme, Renderer>(
        toggler(true)
            .label("Visible")
            .accessible_label("Airplane mode")
            .accessible_description("Disables network radios")
            .accessible_value("On"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Switch).expect("Switch node");
    assert_eq!(node.label(), Some("Airplane mode"));
    assert_eq!(node.description(), Some("Disables network radios"));
    assert_eq!(node.value(), Some("On"));
}

#[test]
fn radio_role() {
    let mut ui = simulator::<(), Theme, Renderer>(radio("Option A", 1, Some(1), |_| ()));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::RadioButton).expect("RadioButton node");
    assert_eq!(node.label(), Some("Option A"));
    assert_eq!(node.is_selected(), Some(true));
    assert_eq!(node.toggled(), Some(accesskit::Toggled::True));
    assert!(
        node.supports_action(accesskit::Action::Click),
        "Radio should advertise Click action"
    );
}

#[test]
fn slider_numeric_range() {
    let mut ui =
        simulator::<(), Theme, Renderer>(slider(0..=100, 50, |_| {}).accessible_label("Volume"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Slider).expect("Slider node");
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(100.0));
    assert_eq!(node.numeric_value(), Some(50.0));
    assert_eq!(node.label(), Some("Volume"));
}

#[test]
fn slider_accessible_metadata() {
    let mut ui = simulator::<(), Theme, Renderer>(
        slider(0..=100, 50, |_| {})
            .accessible_label("Brightness")
            .accessible_description("Screen brightness")
            .accessible_value("Half"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Slider).expect("Slider node");
    assert_eq!(node.label(), Some("Brightness"));
    assert_eq!(node.description(), Some("Screen brightness"));
    assert_eq!(node.value(), Some("Half"));
    assert_eq!(node.numeric_value(), Some(50.0));
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
    let mut ui = simulator::<(), Theme, Renderer>(vertical_slider(0..=100, 50, |_| {}));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Slider).expect("Slider node");
    assert!(node.supports_action(accesskit::Action::Increment));
    assert!(node.supports_action(accesskit::Action::Decrement));
}

#[test]
fn text_input_value_and_label() {
    let mut ui = simulator::<(), Theme, Renderer>(text_input("", "Hello").accessible_label("Name"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::TextInput).expect("TextInput node");
    assert_eq!(node.value(), Some("Hello"));
    assert!(node.label().is_some());
}

#[test]
fn text_input_accessible_metadata_and_set_value() {
    let mut ui = simulator::<String, Theme, Renderer>(
        text_input("", "old")
            .accessible_label("Name")
            .accessible_description("Full name")
            .accessible_value("Display value")
            .on_input(|value| value),
    );
    let tree = ui.accessibility_tree();
    let (input_id, node) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::TextInput)
        .expect("TextInput node");
    assert_eq!(node.label(), Some("Name"));
    assert_eq!(node.description(), Some("Full name"));
    assert_eq!(node.value(), Some("Display value"));
    assert!(node.supports_action(Action::SetValue));

    ui.accessibility_action(&ActionRequest {
        action: Action::SetValue,
        target_tree: TreeId::ROOT,
        target_node: *input_id,
        data: Some(accesskit::ActionData::Value("new".into())),
    });

    let messages: Vec<String> = ui.into_messages().collect();
    assert_eq!(messages, vec!["new"]);
}

#[test]
fn text_editor_label() {
    use iced_widget::text_editor;
    let content = text_editor::Content::new();
    let mut ui =
        simulator::<(), Theme, Renderer>(text_editor(&content).accessible_label("Description"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::TextInput).expect("TextInput node (TextEditor)");
    assert_eq!(node.label(), Some("Description"));
}

#[test]
fn text_editor_disabled_when_no_on_edit() {
    use iced_widget::text_editor;
    let content = text_editor::Content::new();
    let mut ui = simulator::<(), Theme, Renderer>(text_editor(&content).accessible_label("Bio"));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::TextInput).expect("TextInput node (TextEditor)");
    assert!(
        node.is_disabled(),
        "TextEditor without on_edit should be disabled"
    );
    assert!(
        !node.supports_action(accesskit::Action::ReplaceSelectedText),
        "Disabled TextEditor should not advertise ReplaceSelectedText"
    );
}

#[test]
fn text_editor_with_on_edit_has_action() {
    use iced_widget::text_editor;
    let content = text_editor::Content::new();
    let mut ui = simulator::<(), Theme, Renderer>(
        text_editor(&content)
            .accessible_label("Bio")
            .on_action(|_| ()),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::TextInput).expect("TextInput node (TextEditor)");
    assert!(
        !node.is_disabled(),
        "TextEditor with on_edit should be enabled"
    );
    assert!(
        node.supports_action(accesskit::Action::ReplaceSelectedText),
        "Enabled TextEditor should advertise ReplaceSelectedText"
    );
}

#[test]
fn text_editor_accessible_metadata_and_set_value() {
    use iced_widget::text_editor;

    let content = text_editor::Content::new();
    let mut ui = simulator::<i32, Theme, Renderer>(
        text_editor(&content)
            .accessible_label("Bio")
            .accessible_description("Short biography")
            .accessible_value("Displayed biography")
            .on_action(|_| 1),
    );
    let tree = ui.accessibility_tree();
    let (editor_id, node) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::TextInput)
        .expect("TextEditor node");
    assert_eq!(node.label(), Some("Bio"));
    assert_eq!(node.description(), Some("Short biography"));
    assert_eq!(node.value(), Some("Displayed biography"));
    assert!(node.supports_action(Action::SetValue));

    ui.accessibility_action(&ActionRequest {
        action: Action::SetValue,
        target_tree: TreeId::ROOT,
        target_node: *editor_id,
        data: Some(accesskit::ActionData::Value("replacement".into())),
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![1]);
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

#[test]
fn progress_bar_accessible_metadata() {
    let mut ui = simulator::<(), Theme, Renderer>(
        progress_bar(0.0..=100.0, 50.0)
            .accessible_label("Download")
            .accessible_description("Current download progress")
            .accessible_value("Half complete"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::ProgressIndicator).expect("ProgressIndicator node");
    assert_eq!(node.label(), Some("Download"));
    assert_eq!(node.description(), Some("Current download progress"));
    assert_eq!(node.value(), Some("Half complete"));
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
    assert!(tree.nodes.is_empty(), "Space should not produce any nodes");
}

// === COMPOSITIONS ===

#[test]
fn column_contains_children() {
    let mut ui = simulator::<(), Theme, Renderer>(column![
        button("A").on_press(()),
        button("B").on_press(())
    ]);
    let tree = ui.accessibility_tree();
    // Should have: Group (column) + Button A + Label "A" + Button B + Label "B" = 5 nodes
    assert_eq!(tree.nodes.len(), 5, "Column + 2 buttons = 5 nodes");
    assert!(find_node(&tree, Role::GenericContainer).is_some());
    assert!(find_node(&tree, Role::Button).is_some());
}

#[test]
fn row_contains_children() {
    let mut ui =
        simulator::<(), Theme, Renderer>(row![button("A").on_press(()), button("B").on_press(())]);
    let tree = ui.accessibility_tree();
    assert_eq!(tree.nodes.len(), 5, "Row + 2 buttons = 5 nodes");
}

#[test]
fn container_wraps_child() {
    let mut ui = simulator::<(), Theme, Renderer>(container(text("Content")).width(100).height(50));
    let tree = ui.accessibility_tree();
    // Should have: GenericContainer (container) + Label (text) = 2 nodes
    assert_eq!(tree.nodes.len(), 2);
    assert!(find_node(&tree, Role::GenericContainer).is_some());
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
    let tree = ui.accessibility_tree();
    let (button_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Button)
        .expect("Button node");

    let request = ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: *button_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![42],
        "Button click should produce the on_press message"
    );
}

#[test]
fn click_action_on_button_label_dispatches_button_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(button("Press me").on_press(42));
    let tree = ui.accessibility_tree();
    let (_, button_node) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Button)
        .expect("Button node");
    let label_id = button_node
        .children()
        .first()
        .copied()
        .expect("Button label child");

    ui.accessibility_action(&ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: label_id,
        data: None,
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![42]);
}

#[test]
fn click_action_targets_only_requested_button() {
    let mut ui =
        simulator::<i32, Theme, Renderer>(row![button("A").on_press(1), button("B").on_press(2)]);
    let tree = ui.accessibility_tree();
    let (second_button_id, _) = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Button)
        .nth(1)
        .expect("Second button node");

    ui.accessibility_action(&ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: *second_button_id,
        data: None,
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![2]);
}

#[test]
fn focus_action_updates_tree_focus_for_controls() {
    let mut ui = simulator::<(), Theme, Renderer>(column![
        button("Button").on_press(()),
        checkbox(false).label("Checkbox").on_toggle(|_| ()),
        toggler(false).label("Toggle").on_toggle(|_| ()),
        radio("Radio", 1, Some(0), |_| ()),
        slider(0..=100, 50, |_| ()).accessible_label("Slider"),
        text_input("Placeholder", "").on_input(|_| ()),
        pick_list(Some("A"), ["A", "B"].as_slice(), |s: &&str| {
            s.to_string()
        })
        .on_select(|_| ()),
    ]);

    for role in [
        Role::Button,
        Role::CheckBox,
        Role::Switch,
        Role::RadioButton,
        Role::Slider,
        Role::TextInput,
        Role::ComboBox,
    ] {
        let tree = ui.accessibility_tree();
        let (target_id, _) = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == role)
            .unwrap_or_else(|| panic!("Expected {role:?} node"));

        ui.accessibility_action(&ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: *target_id,
            data: None,
        });

        let tree = ui.accessibility_tree();
        assert_eq!(tree.focus, *target_id, "{role:?} should receive focus");
    }
}

#[test]
fn focus_action_on_button_label_focuses_button() {
    let mut ui = simulator::<(), Theme, Renderer>(button("Press").on_press(()));
    let tree = ui.accessibility_tree();
    let (button_id, button_node) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Button)
        .expect("Button node");
    let label_id = button_node
        .children()
        .first()
        .copied()
        .expect("Button label child");

    ui.accessibility_action(&ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: label_id,
        data: None,
    });

    let tree = ui.accessibility_tree();
    assert_eq!(tree.focus, *button_id);
}

#[test]
fn tab_focus_updates_accessibility_focus() {
    let mut ui =
        simulator::<(), Theme, Renderer>(row![button("A").on_press(()), button("B").on_press(())]);

    let before = ui.accessibility_tree();
    let first_button = before
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Button)
        .map(|(id, _)| *id)
        .expect("First button node");

    let status = ui.tap_key(keyboard::Key::Named(keyboard::key::Named::Tab));
    assert_eq!(status, iced_test::core::event::Status::Captured);

    let after = ui.accessibility_tree();
    assert_eq!(after.focus, first_button);
}

#[test]
fn repeated_tab_advances_accessibility_focus() {
    let mut ui =
        simulator::<(), Theme, Renderer>(row![button("A").on_press(()), button("B").on_press(())]);

    let tree = ui.accessibility_tree();
    let second_button = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Button)
        .nth(1)
        .map(|(id, _)| *id)
        .expect("Second button node");

    let _ = ui.tap_key(keyboard::Key::Named(keyboard::key::Named::Tab));
    let _ = ui.tap_key(keyboard::Key::Named(keyboard::key::Named::Tab));

    let after = ui.accessibility_tree();
    assert_eq!(after.focus, second_button);
}

#[test]
fn shift_tab_moves_focus_backward() {
    let mut ui =
        simulator::<(), Theme, Renderer>(row![button("A").on_press(()), button("B").on_press(())]);

    let tree = ui.accessibility_tree();
    let second_button = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Button)
        .nth(1)
        .map(|(id, _)| *id)
        .expect("Second button node");

    let tab = keyboard::Key::Named(keyboard::key::Named::Tab);
    let _ = ui.simulate([Event::Keyboard(keyboard::Event::KeyPressed {
        key: tab.clone(),
        modified_key: tab,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::SHIFT,
        repeat: false,
        text: None,
    })]);

    let after = ui.accessibility_tree();
    assert_eq!(after.focus, second_button);
}

#[test]
fn focused_button_activates_with_enter_and_space() {
    let mut ui = simulator::<i32, Theme, Renderer>(button("Press").on_press(7));

    let _ = ui.tap_key(keyboard::Key::Named(keyboard::key::Named::Tab));
    let _ = ui.tap_key(keyboard::Key::Named(keyboard::key::Named::Enter));
    let _ = ui.tap_key(keyboard::Key::Named(keyboard::key::Named::Space));

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![7, 7]);
}

#[test]
fn increment_action_dispatches_slider_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(slider(0..=100, 50, |v| v));
    // First get the tree to find the slider's node ID
    let tree = ui.accessibility_tree();
    let (slider_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Slider)
        .expect("Slider node");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Increment,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *slider_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![51],
        "Slider increment with step=1 should produce 51"
    );
}

#[test]
fn decrement_action_dispatches_slider_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(slider(0..=100, 50, |v| v));
    let tree = ui.accessibility_tree();
    let (slider_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Slider)
        .expect("Slider node");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Decrement,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *slider_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![49],
        "Slider decrement with step=1 should produce 49"
    );
}

#[test]
fn set_value_action_dispatches_slider_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(slider(0..=100, 50, |v| v));
    let tree = ui.accessibility_tree();
    let (slider_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Slider)
        .expect("Slider node");

    ui.accessibility_action(&ActionRequest {
        action: Action::SetValue,
        target_tree: TreeId::ROOT,
        target_node: *slider_id,
        data: Some(accesskit::ActionData::NumericValue(42.0)),
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![42]);
}

#[test]
fn set_value_action_clamps_slider_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(slider(0..=100, 50, |v| v));
    let tree = ui.accessibility_tree();
    let (slider_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Slider)
        .expect("Slider node");

    ui.accessibility_action(&ActionRequest {
        action: Action::SetValue,
        target_tree: TreeId::ROOT,
        target_node: *slider_id,
        data: Some(accesskit::ActionData::NumericValue(250.0)),
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![100]);
}

#[test]
fn set_value_action_dispatches_vertical_slider_message() {
    let mut ui = simulator::<i32, Theme, Renderer>(vertical_slider(0..=100, 50, |v| v));
    let tree = ui.accessibility_tree();
    let (slider_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Slider)
        .expect("Slider node");

    ui.accessibility_action(&ActionRequest {
        action: Action::SetValue,
        target_tree: TreeId::ROOT,
        target_node: *slider_id,
        data: Some(accesskit::ActionData::Value("75".into())),
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![75]);
}

#[test]
fn slider_increment_targets_only_requested_slider() {
    let mut ui = simulator::<i32, Theme, Renderer>(row![
        slider(0..=100, 10, |v| v),
        slider(0..=100, 50, |v| v + 1000),
    ]);
    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Slider)
        .nth(1)
        .expect("Second Slider node");

    ui.accessibility_action(&ActionRequest {
        action: Action::Increment,
        target_tree: TreeId::ROOT,
        target_node: *target_id,
        data: None,
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![1051]);
}

// === MENU / DROPDOWN ACCESSIBILITY ===

#[test]
fn picklist_menu_items_visible_when_open() {
    let options = vec!["Option A", "Option B", "Option C"];
    let mut ui = simulator::<(), Theme, Renderer>(
        pick_list(Some("Option A"), options.as_slice(), |s: &&str| {
            s.to_string()
        })
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

    assert_eq!(
        items.len(),
        3,
        "Should have 3 MenuItem nodes when dropdown is open"
    );
    assert_eq!(items[0].label(), Some("Option A"));
    assert!(items[0].supports_action(accesskit::Action::Click));
}

#[test]
fn picklist_expand_action_exposes_menu_items() {
    let options = vec!["Option A", "Option B", "Option C"];
    let mut ui = simulator::<(), Theme, Renderer>(
        pick_list(Some("Option A"), options.as_slice(), |s: &&str| {
            s.to_string()
        })
        .on_select(|_| ()),
    );

    let tree = ui.accessibility_tree();
    let (combo_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::ComboBox)
        .expect("PickList ComboBox node");

    ui.accessibility_action(&ActionRequest {
        action: Action::Expand,
        target_tree: TreeId::ROOT,
        target_node: *combo_id,
        data: None,
    });

    let tree = ui.accessibility_tree();
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::MenuItem)
            .count(),
        3
    );
}

#[test]
fn picklist_menu_item_click_dispatches_selection() {
    let options = vec!["Alpha", "Beta", "Gamma"];
    let mut ui = simulator::<String, Theme, Renderer>(
        pick_list(Some("Alpha"), options.as_slice(), |s: &&str| s.to_string())
            .on_select(|s| s.to_string()),
    );

    // Open the dropdown
    ui.point_at(Point::new(10.0, 10.0));
    let _ = ui.simulate(iced_test::simulator::click());

    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::MenuItem && n.label() == Some("Beta"))
        .expect("Second menu item 'Beta' should exist");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *target_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<String> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec!["Beta"],
        "Clicking menu item should produce its selected message"
    );
}

#[test]
fn pick_list_disabled_when_no_on_select() {
    let options = vec!["A", "B"];
    let mut ui =
        simulator::<(), Theme, Renderer>(pick_list(Some("A"), options.as_slice(), |s: &&str| {
            s.to_string()
        }));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::ComboBox).expect("PickList ComboBox node");
    assert!(
        node.is_disabled(),
        "PickList without on_select should be disabled"
    );
}

// === ACCESSIBLE LABEL ON GRAPHIC WIDGETS ===

#[test]
fn image_with_accessible_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        iced_widget::image("test.png")
            .accessible_label("Company Logo")
            .accessible_description("Square brand mark"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Image).expect("Image node");
    assert_eq!(node.label(), Some("Company Logo"));
    assert_eq!(node.description(), Some("Square brand mark"));
}

#[test]
fn svg_with_accessible_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        iced_widget::svg(iced_widget::core::svg::Handle::from_path("icon.svg"))
            .accessible_label("Settings Icon")
            .accessible_description("Cog icon"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Image).expect("Svg node");
    assert_eq!(node.label(), Some("Settings Icon"));
    assert_eq!(node.description(), Some("Cog icon"));
}

#[test]
fn progress_bar_with_accessible_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        progress_bar(0.0..=100.0, 50.0).accessible_label("Download Progress"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::ProgressIndicator).expect("ProgressIndicator node");
    assert_eq!(node.label(), Some("Download Progress"));
}

// === QRCODE CUSTOMIZABLE LABEL ===

#[test]
fn qr_code_custom_label_and_value() {
    use iced_widget::qr_code::Data;
    let data = Data::new("https://example.com").unwrap();
    let mut ui = simulator::<(), Theme, Renderer>(
        iced_widget::qr_code(&data)
            .accessible_label("Example QR")
            .accessible_description("Encodes the example website")
            .encoded_value("https://example.com"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Image).expect("QRCode node");
    assert_eq!(node.label(), Some("Example QR"));
    assert_eq!(node.description(), Some("Encodes the example website"));
    assert_eq!(node.value(), Some("https://example.com"));
}

// === TOOLTIP HAS POPUP ===

#[test]
fn tooltip_has_popup() {
    let mut ui = simulator::<(), Theme, Renderer>(tooltip(
        text("Hover me"),
        text("Tooltip text"),
        tooltip::Position::Top,
    ));
    let tree = ui.accessibility_tree();
    let node = tree
        .nodes
        .iter()
        .find(|(_, n)| n.has_popup() == Some(accesskit::HasPopup::Menu))
        .map(|(_, n)| n)
        .expect("Tooltip popup node");
    // Should signal to AT that a popup (tooltip) exists
    assert_eq!(node.has_popup(), Some(accesskit::HasPopup::Menu));
}

// === COMBOBOX DROPDOWN MENU ITEMS ===

#[test]
fn combo_box_menu_items_visible_when_focused() {
    use iced_widget::combo_box;

    let state = combo_box::State::new(vec!["A", "B", "C"]);
    let mut ui =
        simulator::<(), Theme, Renderer>(combo_box(&state, "Pick...", None::<&&str>, |_: &str| ()));

    // Click to focus and open the dropdown
    ui.point_at(Point::new(10.0, 10.0));
    let _ = ui.simulate(iced_test::simulator::click());

    let tree = ui.accessibility_tree();
    let items: Vec<&accesskit::Node> = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::MenuItem)
        .map(|(_, n)| n)
        .collect();
    assert_eq!(
        items.len(),
        3,
        "Should have 3 MenuItem nodes when dropdown is open"
    );
    assert_eq!(items[0].label(), Some("A"));
    assert!(items[0].supports_action(accesskit::Action::Click));
}

#[test]
fn combo_box_expand_action_exposes_menu_items() {
    use iced_widget::combo_box;

    let state = combo_box::State::new(vec!["A", "B", "C"]);
    let mut ui =
        simulator::<(), Theme, Renderer>(combo_box(&state, "Pick...", None::<&&str>, |_: &str| ()));

    let tree = ui.accessibility_tree();
    let (combo_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::ComboBox)
        .expect("ComboBox node");

    ui.accessibility_action(&ActionRequest {
        action: Action::Expand,
        target_tree: TreeId::ROOT,
        target_node: *combo_id,
        data: None,
    });

    let tree = ui.accessibility_tree();
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::MenuItem)
            .count(),
        3
    );
}

#[test]
fn combo_box_menu_item_click_dispatches_selection() {
    use iced_widget::combo_box;

    let state = combo_box::State::new(vec!["Alpha", "Beta", "Gamma"]);
    let mut ui = simulator::<&str, Theme, Renderer>(combo_box(
        &state,
        "Pick...",
        None::<&&str>,
        |s: &str| s,
    ));

    // Open the dropdown
    ui.point_at(Point::new(10.0, 10.0));
    let _ = ui.simulate(iced_test::simulator::click());

    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::MenuItem && n.label() == Some("Beta"))
        .expect("Second menu item 'Beta' should exist");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *target_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<&str> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec!["Beta"],
        "Clicking ComboBox item should produce its selected message"
    );
}

#[test]
fn combo_box_has_popup() {
    use iced_widget::combo_box;
    let state = combo_box::State::new(vec!["A", "B", "C"]);
    let mut ui =
        simulator::<(), Theme, Renderer>(combo_box(&state, "Pick...", None::<&&str>, |_: &str| ()));
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::ComboBox).expect("ComboBox node");
    assert_eq!(node.has_popup(), Some(accesskit::HasPopup::Listbox));
}

// === TABLE ACCESSIBILITY ===

#[test]
fn table_roles_and_indices() {
    use iced_widget::table;

    let columns = [table::column(text("Name"), |s: String| text(s))];
    let rows = vec!["Alice".to_string(), "Bob".to_string()];
    let mut ui = simulator::<(), Theme, Renderer>(
        table::table(columns, rows).width(iced_test::core::Length::Fixed(200.0)),
    );

    let tree = ui.accessibility_tree();
    assert!(
        find_node(&tree, Role::Grid).is_some(),
        "Grid node should exist"
    );
    let table_node = find_node(&tree, Role::Grid).unwrap();
    // row_count includes the header row + data rows
    assert_eq!(table_node.row_count(), Some(3));
    assert_eq!(table_node.column_count(), Some(1));

    let row_nodes: Vec<&accesskit::Node> = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::TreeItem)
        .map(|(_, n)| n)
        .collect();
    assert_eq!(
        row_nodes.len(),
        3,
        "Should have 3 TreeItem row nodes (1 header + 2 data)"
    );
    assert!(row_nodes.iter().all(|node| has_non_empty_bounds(node)));
    assert_eq!(row_nodes[0].row_index(), Some(0));
    assert_eq!(row_nodes[1].row_index(), Some(1));
    assert_eq!(row_nodes[2].row_index(), Some(2));

    let cells: Vec<&accesskit::Node> = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Cell)
        .map(|(_, n)| n)
        .collect();
    assert_eq!(cells.len(), 2, "Should have 2 Cell nodes (data rows)");

    let headers: Vec<&accesskit::Node> = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::ColumnHeader)
        .map(|(_, n)| n)
        .collect();
    assert_eq!(headers.len(), 1, "Should have 1 ColumnHeader node");
    assert_eq!(headers[0].label(), Some("Name"));
    assert_eq!(headers[0].row_index(), Some(0));
    assert_eq!(headers[0].column_index(), Some(0));
    assert_eq!(cells[0].label(), Some("Alice"));
    assert_eq!(cells[0].row_index(), Some(1));
    assert_eq!(cells[0].column_index(), Some(0));
}

// === ACTION DISPATCH FOR CHECKBOX / TOGGLER / RADIO ===

#[test]
fn checkbox_click_dispatches_toggle() {
    let mut ui =
        simulator::<bool, Theme, Renderer>(checkbox(false).label("Accept").on_toggle(|v| v));
    let tree = ui.accessibility_tree();
    let (box_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::CheckBox)
        .expect("CheckBox node");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *box_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<bool> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![true],
        "Checkbox click should produce the toggle message (true)"
    );
}

#[test]
fn checkbox_click_targets_only_requested_control() {
    let mut ui = simulator::<i32, Theme, Renderer>(row![
        checkbox(false).label("A").on_toggle(|_| 1),
        checkbox(false).label("B").on_toggle(|_| 2),
    ]);
    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::CheckBox)
        .nth(1)
        .expect("Second CheckBox node");

    ui.accessibility_action(&ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: *target_id,
        data: None,
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![2]);
}

#[test]
fn toggler_click_dispatches_toggle() {
    let mut ui = simulator::<bool, Theme, Renderer>(toggler(false).label("WiFi").on_toggle(|v| v));
    let tree = ui.accessibility_tree();
    let (tog_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Switch)
        .expect("Switch node");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *tog_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<bool> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![true],
        "Toggler click should produce the toggle message (true)"
    );
}

#[test]
fn toggler_click_targets_only_requested_control() {
    let mut ui = simulator::<i32, Theme, Renderer>(row![
        toggler(false).label("A").on_toggle(|_| 1),
        toggler(false).label("B").on_toggle(|_| 2),
    ]);
    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Switch)
        .nth(1)
        .expect("Second Switch node");

    ui.accessibility_action(&ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: *target_id,
        data: None,
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![2]);
}

#[test]
fn radio_click_dispatches_selection() {
    let mut ui = simulator::<i32, Theme, Renderer>(radio("Option A", 1, Some(2), |v| v));
    let tree = ui.accessibility_tree();
    let (radio_id, _) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::RadioButton)
        .expect("RadioButton node");

    let request = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *radio_id,
        data: None,
    };
    ui.accessibility_action(&request);
    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![1],
        "Radio click should produce the selected value (1)"
    );
}

#[test]
fn radio_click_targets_only_requested_control() {
    let mut ui = simulator::<i32, Theme, Renderer>(row![
        radio("A", 1, Some(0), |v| v),
        radio("B", 2, Some(0), |v| v),
    ]);
    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::RadioButton)
        .nth(1)
        .expect("Second RadioButton node");

    ui.accessibility_action(&ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: *target_id,
        data: None,
    });

    let messages: Vec<i32> = ui.into_messages().collect();
    assert_eq!(messages, vec![2]);
}

// === FOCUS TRACKING ===

#[test]
fn focus_tracking_focus_exists() {
    let mut ui = simulator::<(), Theme, Renderer>(text_input("", "Hello"));

    let tree = ui.accessibility_tree();
    // Verify focus always points to an existing node (even if it's the root)
    assert!(
        tree.nodes.iter().any(|(id, _)| *id == tree.focus),
        "Focus should point to an existing node"
    );
}

#[test]
fn focus_tracking_text_input_sets_focus_flag() {
    let mut ui = simulator::<(), Theme, Renderer>(text_input("", "Hello"));

    // text_input doesn't handle Focus action directly,
    // but it does set set_accesskit_focused(true) when its internal state is focused.
    // The tree.focus falls back to root when nothing is focused.
    let tree = ui.accessibility_tree();
    let root = tree.tree.as_ref().unwrap().root;
    assert_eq!(tree.focus, root, "Initially focus defaults to root");
}

// === SCROLLABLE SCROLL ACTIONS ===

#[test]
fn scrollable_with_overflow_shows_actions() {
    use iced_test::core::Length;

    // Force content height far larger than scrollable viewport
    let mut ui = simulator::<(), Theme, Renderer>(
        scrollable(column![text("Content that scrolls"),].height(Length::Fixed(1000.0)))
            .height(Length::Fixed(50.0)),
    );

    let tree = ui.accessibility_tree();
    // Find the scrollable Group node (not the root) by filtering for one
    // with scroll actions or non-zero scroll_y_max
    let node = tree
        .nodes
        .iter()
        .find_map(|(_, n)| {
            if n.role() == Role::Group && n.supports_action(accesskit::Action::ScrollDown) {
                Some(n)
            } else {
                None
            }
        })
        .expect("Scrollable Group node with ScrollDown action");
    assert!(
        node.supports_action(accesskit::Action::ScrollUp),
        "Overflowing scrollable should advertise ScrollUp"
    );
    assert!(node.scroll_y_max() > Some(0.0));
}

#[test]
fn scrollable_scroll_into_view_moves_to_descendant() {
    use iced_test::core::Length;

    let mut ui = simulator::<(), Theme, Renderer>(
        scrollable(column![
            text("Top"),
            iced_widget::space::Space::new().height(Length::Fixed(900.0)),
            text("Target")
        ])
        .height(Length::Fixed(50.0)),
    );

    let tree = ui.accessibility_tree();
    let (target_id, _) = tree
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.value() == Some("Target"))
        .expect("Target label node");

    ui.accessibility_action(&ActionRequest {
        action: Action::ScrollIntoView,
        target_tree: TreeId::ROOT,
        target_node: *target_id,
        data: None,
    });

    let tree = ui.accessibility_tree();
    let scrollable = tree
        .nodes
        .iter()
        .find_map(|(_, node)| {
            if node.role() == Role::Group && node.scroll_y_max() > Some(0.0) {
                Some(node)
            } else {
                None
            }
        })
        .expect("Scrollable Group node");

    assert!(scrollable.scroll_y() > Some(0.0));
}

#[test]
fn scrollable_without_overflow_no_scroll_actions() {
    // Single line of text fills the scrollable without overflow
    let mut ui = simulator::<(), Theme, Renderer>(scrollable(text("Short content")));

    let tree = ui.accessibility_tree();
    if let Some(node) = find_node(&tree, Role::Group) {
        assert!(
            !node.supports_action(accesskit::Action::ScrollDown),
            "Non-overflowing scrollable should NOT advertise ScrollDown"
        );
        assert!(
            !node.supports_action(accesskit::Action::ScrollUp),
            "Non-overflowing scrollable should NOT advertise ScrollUp"
        );
        assert_eq!(node.scroll_y_max(), Some(0.0));
    }
}
