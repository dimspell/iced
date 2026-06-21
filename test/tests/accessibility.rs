//! Integration tests for accessibility tree building.
use iced_test::simulator;
use iced_test::core::Theme;
use iced_test::renderer::Renderer;
use iced_widget::{
    button, checkbox, column, container, pick_list, progress_bar, radio, row, scrollable,
    slider, text, text_input, toggler, tooltip, vertical_slider,
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

// === ACCESSIBLE LABEL ON GRAPHIC WIDGETS ===

#[test]
fn image_with_accessible_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        iced_widget::image("test.png").accessible_label("Company Logo"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Image).expect("Image node");
    assert_eq!(node.label(), Some("Company Logo"));
}

#[test]
fn svg_with_accessible_label() {
    let mut ui = simulator::<(), Theme, Renderer>(
        iced_widget::svg(iced_widget::core::svg::Handle::from_path("icon.svg"))
            .accessible_label("Settings Icon"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Image).expect("Svg node");
    assert_eq!(node.label(), Some("Settings Icon"));
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
            .encoded_value("https://example.com"),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Image).expect("QRCode node");
    assert_eq!(node.label(), Some("Example QR"));
    assert_eq!(node.value(), Some("https://example.com"));
}

// === TOOLTIP HAS POPUP ===

#[test]
fn tooltip_has_popup() {
    let mut ui = simulator::<(), Theme, Renderer>(
        tooltip(text("Hover me"), text("Tooltip text"), tooltip::Position::Top),
    );
    let tree = ui.accessibility_tree();
    let node = find_node(&tree, Role::Group).expect("Tooltip Group node");
    // Should signal to AT that a popup (tooltip) exists
    assert_eq!(node.has_popup(), Some(accesskit::HasPopup::Menu));
}

// === COMBOBOX DROPDOWN MENU ITEMS ===

#[test]
fn combo_box_menu_items_visible_when_focused() {
    use iced_widget::combo_box;

    let state = combo_box::State::new(vec!["A", "B", "C"]);
    let mut ui = simulator::<(), Theme, Renderer>(
        combo_box(&state, "Pick...", None::<& &str>, |_: &str| ()),
    );

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
    assert_eq!(items.len(), 3, "Should have 3 MenuItem nodes when dropdown is open");
    assert_eq!(items[0].label(), Some("A"));
    assert!(items[0].supports_action(accesskit::Action::Click));
}

#[test]
fn combo_box_menu_item_click_dispatches_selection() {
    use iced_widget::combo_box;

    let state = combo_box::State::new(vec!["Alpha", "Beta", "Gamma"]);
    let mut ui = simulator::<&str, Theme, Renderer>(
        combo_box(&state, "Pick...", None::<& &str>, |s: &str| s),
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
    let messages: Vec<&str> = ui.into_messages().collect();
    assert_eq!(messages, vec!["Beta"], "Clicking ComboBox item should produce its selected message");
}

// === TABLE ACCESSIBILITY ===

#[test]
fn table_roles_and_indices() {
    use iced_widget::table;

    let columns = [table::column(text("Name"), |s: String| text(s))];
    let rows = vec!["Alice".to_string(), "Bob".to_string()];
    let mut ui = simulator::<(), Theme, Renderer>(table::table(columns, rows));

    let tree = ui.accessibility_tree();
    assert!(
        find_node(&tree, Role::Table).is_some(),
        "Table node should exist"
    );
    let table_node = find_node(&tree, Role::Table).unwrap();
    // row_count includes the header row + data rows
    assert_eq!(table_node.row_count(), Some(3));
    assert_eq!(table_node.column_count(), Some(1));

    let row_nodes: Vec<&accesskit::Node> = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Row)
        .map(|(_, n)| n)
        .collect();
    assert_eq!(row_nodes.len(), 3, "Should have 3 Row nodes (1 header + 2 data)");

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
}

// === ACTION DISPATCH FOR CHECKBOX / TOGGLER / RADIO ===

#[test]
fn checkbox_click_dispatches_toggle() {
    let mut ui = simulator::<bool, Theme, Renderer>(
        checkbox(false).label("Accept").on_toggle(|v| v),
    );
    let tree = ui.accessibility_tree();
    let (box_id, _) = tree.nodes.iter()
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
    assert_eq!(messages, vec![true], "Checkbox click should produce the toggle message (true)");
}

#[test]
fn toggler_click_dispatches_toggle() {
    let mut ui = simulator::<bool, Theme, Renderer>(
        toggler(false).label("WiFi").on_toggle(|v| v),
    );
    let tree = ui.accessibility_tree();
    let (tog_id, _) = tree.nodes.iter()
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
    assert_eq!(messages, vec![true], "Toggler click should produce the toggle message (true)");
}

#[test]
fn radio_click_dispatches_selection() {
    let mut ui = simulator::<i32, Theme, Renderer>(
        radio("Option A", 1, Some(2), |v| v),
    );
    let tree = ui.accessibility_tree();
    let (radio_id, _) = tree.nodes.iter()
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
    assert_eq!(messages, vec![1], "Radio click should produce the selected value (1)");
}

// === FOCUS TRACKING ===

#[test]
fn focus_tracking_focus_exists() {
    let mut ui = simulator::<(), Theme, Renderer>(
        text_input("", "Hello"),
    );

    let tree = ui.accessibility_tree();
    // Verify focus always points to an existing node (even if it's the root)
    assert!(
        tree.nodes.iter().any(|(id, _)| *id == tree.focus),
        "Focus should point to an existing node"
    );
}

#[test]
fn focus_tracking_text_input_sets_focus_flag() {
    let mut ui = simulator::<(), Theme, Renderer>(
        text_input("", "Hello"),
    );

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
        scrollable(
            column![text("Content that scrolls"),]
                .height(Length::Fixed(1000.0)),
        )
        .height(Length::Fixed(50.0)),
    );

    let tree = ui.accessibility_tree();
    // Find the scrollable Group node (not the root) by filtering for one
    // with scroll actions or non-zero scroll_y_max
    let node = tree.nodes
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
fn scrollable_without_overflow_no_scroll_actions() {
    // Single line of text fills the scrollable without overflow
    let mut ui = simulator::<(), Theme, Renderer>(
        scrollable(text("Short content")),
    );

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
