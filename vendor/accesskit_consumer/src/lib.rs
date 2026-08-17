// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![no_std]

extern crate alloc;

pub(crate) mod tree;
pub use tree::{ChangeHandler as TreeChangeHandler, State as TreeState, Tree};

pub(crate) mod node;
pub use node::{Node, NodeId};

pub(crate) mod filters;
pub use filters::{FilterResult, common_filter, common_filter_with_root_exception};

pub(crate) mod iterators;

pub(crate) mod text;
pub use text::{
    Position as TextPosition, Range as TextRange, RangePropertyValue as TextRangePropertyValue,
    WeakRange as WeakTextRange,
};

#[cfg(test)]
mod tests {
    use accesskit::{
        Affine, Node, NodeId as LocalNodeId, Rect, Role, Tree, TreeId, TreeUpdate, Vec2,
    };
    use alloc::vec;

    use crate::FilterResult;
    use crate::node::NodeId;
    use crate::tree::TreeIndex;

    pub fn nid(id: LocalNodeId) -> NodeId {
        NodeId::new(id, TreeIndex(0))
    }

    pub const ROOT_ID: LocalNodeId = LocalNodeId(0);
    pub const PARAGRAPH_0_ID: LocalNodeId = LocalNodeId(1);
    pub const LABEL_0_0_IGNORED_ID: LocalNodeId = LocalNodeId(2);
    pub const PARAGRAPH_1_IGNORED_ID: LocalNodeId = LocalNodeId(3);
    pub const BUTTON_1_0_HIDDEN_ID: LocalNodeId = LocalNodeId(4);
    pub const CONTAINER_1_0_0_HIDDEN_ID: LocalNodeId = LocalNodeId(5);
    pub const LABEL_1_1_ID: LocalNodeId = LocalNodeId(6);
    pub const BUTTON_1_2_HIDDEN_ID: LocalNodeId = LocalNodeId(7);
    pub const CONTAINER_1_2_0_HIDDEN_ID: LocalNodeId = LocalNodeId(8);
    pub const PARAGRAPH_2_ID: LocalNodeId = LocalNodeId(9);
    pub const LABEL_2_0_ID: LocalNodeId = LocalNodeId(10);
    pub const PARAGRAPH_3_IGNORED_ID: LocalNodeId = LocalNodeId(11);
    pub const EMPTY_CONTAINER_3_0_IGNORED_ID: LocalNodeId = LocalNodeId(12);
    pub const LINK_3_1_IGNORED_ID: LocalNodeId = LocalNodeId(13);
    pub const LABEL_3_1_0_ID: LocalNodeId = LocalNodeId(14);
    pub const BUTTON_3_2_ID: LocalNodeId = LocalNodeId(15);
    pub const EMPTY_CONTAINER_3_3_IGNORED_ID: LocalNodeId = LocalNodeId(16);

    pub fn test_tree() -> crate::tree::Tree {
        let root = {
            let mut node = Node::new(Role::RootWebArea);
            node.set_children(vec![
                PARAGRAPH_0_ID,
                PARAGRAPH_1_IGNORED_ID,
                PARAGRAPH_2_ID,
                PARAGRAPH_3_IGNORED_ID,
            ]);
            node
        };
        let paragraph_0 = {
            let mut node = Node::new(Role::Paragraph);
            node.set_children(vec![LABEL_0_0_IGNORED_ID]);
            node
        };
        let label_0_0_ignored = {
            let mut node = Node::new(Role::Label);
            node.set_value("label_0_0_ignored");
            node
        };
        let paragraph_1_ignored = {
            let mut node = Node::new(Role::Paragraph);
            node.set_transform(Affine::translate(Vec2::new(10.0, 40.0)));
            node.set_bounds(Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 800.0,
                y1: 40.0,
            });
            node.set_children(vec![
                BUTTON_1_0_HIDDEN_ID,
                LABEL_1_1_ID,
                BUTTON_1_2_HIDDEN_ID,
            ]);
            node
        };
        let button_1_0_hidden = {
            let mut node = Node::new(Role::Button);
            node.set_label("button_1_0_hidden");
            node.set_hidden();
            node.set_children(vec![CONTAINER_1_0_0_HIDDEN_ID]);
            node
        };
        let container_1_0_0_hidden = {
            let mut node = Node::new(Role::GenericContainer);
            node.set_hidden();
            node
        };
        let label_1_1 = {
            let mut node = Node::new(Role::Label);
            node.set_bounds(Rect {
                x0: 10.0,
                y0: 10.0,
                x1: 90.0,
                y1: 30.0,
            });
            node.set_value("label_1_1");
            node
        };
        let button_1_2_hidden = {
            let mut node = Node::new(Role::Button);
            node.set_label("button_1_2_hidden");
            node.set_hidden();
            node.set_children(vec![CONTAINER_1_2_0_HIDDEN_ID]);
            node
        };
        let container_1_2_0_hidden = {
            let mut node = Node::new(Role::GenericContainer);
            node.set_hidden();
            node
        };
        let paragraph_2 = {
            let mut node = Node::new(Role::Paragraph);
            node.set_children(vec![LABEL_2_0_ID]);
            node
        };
        let label_2_0 = {
            let mut node = Node::new(Role::Label);
            node.set_label("label_2_0");
            node
        };
        let paragraph_3_ignored = {
            let mut node = Node::new(Role::Paragraph);
            node.set_children(vec![
                EMPTY_CONTAINER_3_0_IGNORED_ID,
                LINK_3_1_IGNORED_ID,
                BUTTON_3_2_ID,
                EMPTY_CONTAINER_3_3_IGNORED_ID,
            ]);
            node
        };
        let empty_container_3_0_ignored = Node::new(Role::GenericContainer);
        let link_3_1_ignored = {
            let mut node = Node::new(Role::Link);
            node.set_children(vec![LABEL_3_1_0_ID]);
            node
        };
        let label_3_1_0 = {
            let mut node = Node::new(Role::Label);
            node.set_value("label_3_1_0");
            node
        };
        let button_3_2 = {
            let mut node = Node::new(Role::Button);
            node.set_label("button_3_2");
            node
        };
        let empty_container_3_3_ignored = Node::new(Role::GenericContainer);
        let initial_update = TreeUpdate {
            nodes: vec![
                (ROOT_ID, root),
                (PARAGRAPH_0_ID, paragraph_0),
                (LABEL_0_0_IGNORED_ID, label_0_0_ignored),
                (PARAGRAPH_1_IGNORED_ID, paragraph_1_ignored),
                (BUTTON_1_0_HIDDEN_ID, button_1_0_hidden),
                (CONTAINER_1_0_0_HIDDEN_ID, container_1_0_0_hidden),
                (LABEL_1_1_ID, label_1_1),
                (BUTTON_1_2_HIDDEN_ID, button_1_2_hidden),
                (CONTAINER_1_2_0_HIDDEN_ID, container_1_2_0_hidden),
                (PARAGRAPH_2_ID, paragraph_2),
                (LABEL_2_0_ID, label_2_0),
                (PARAGRAPH_3_IGNORED_ID, paragraph_3_ignored),
                (EMPTY_CONTAINER_3_0_IGNORED_ID, empty_container_3_0_ignored),
                (LINK_3_1_IGNORED_ID, link_3_1_ignored),
                (LABEL_3_1_0_ID, label_3_1_0),
                (BUTTON_3_2_ID, button_3_2),
                (EMPTY_CONTAINER_3_3_IGNORED_ID, empty_container_3_3_ignored),
            ],
            tree: Some(Tree::new(ROOT_ID)),
            tree_id: TreeId::ROOT,
            focus: ROOT_ID,
        };
        crate::tree::Tree::new(initial_update, false)
    }

    pub fn test_tree_filter(node: &crate::Node) -> FilterResult {
        let id = node.id();
        if node.is_hidden() {
            FilterResult::ExcludeSubtree
        } else if id == nid(LABEL_0_0_IGNORED_ID)
            || id == nid(PARAGRAPH_1_IGNORED_ID)
            || id == nid(PARAGRAPH_3_IGNORED_ID)
            || id == nid(EMPTY_CONTAINER_3_0_IGNORED_ID)
            || id == nid(LINK_3_1_IGNORED_ID)
            || id == nid(EMPTY_CONTAINER_3_3_IGNORED_ID)
        {
            FilterResult::ExcludeNode
        } else {
            FilterResult::Include
        }
    }

    #[test]
    fn column_index_round_trip() {
        const TABLE_ID: LocalNodeId = LocalNodeId(100);
        const HEADER_ROW_ID: LocalNodeId = LocalNodeId(101);
        const COL_A_ID: LocalNodeId = LocalNodeId(102);
        const COL_B_ID: LocalNodeId = LocalNodeId(103);
        const DATA_ROW_ID: LocalNodeId = LocalNodeId(104);
        const CELL_A_ID: LocalNodeId = LocalNodeId(105);
        const CELL_B_ID: LocalNodeId = LocalNodeId(106);

        let mut table = Node::new(Role::Table);
        table.set_row_count(2);
        table.set_column_count(2);
        table.set_children(vec![HEADER_ROW_ID, DATA_ROW_ID]);

        let mut header_row = Node::new(Role::Row);
        header_row.set_row_index(0);
        header_row.set_children(vec![COL_A_ID, COL_B_ID]);

        let mut col_a = Node::new(Role::ColumnHeader);
        col_a.set_column_index(0);
        col_a.set_label("Name");

        let mut col_b = Node::new(Role::ColumnHeader);
        col_b.set_column_index(1);
        col_b.set_label("Value");

        let mut data_row = Node::new(Role::Row);
        data_row.set_row_index(1);
        data_row.set_children(vec![CELL_A_ID, CELL_B_ID]);

        let mut cell_a = Node::new(Role::Cell);
        cell_a.set_column_index(0);
        cell_a.set_row_index(1);
        cell_a.set_value("foo");

        let mut cell_b = Node::new(Role::Cell);
        cell_b.set_column_index(1);
        cell_b.set_row_index(1);
        cell_b.set_value("bar");

        let update = TreeUpdate {
            nodes: vec![
                (TABLE_ID, table),
                (HEADER_ROW_ID, header_row),
                (COL_A_ID, col_a),
                (COL_B_ID, col_b),
                (DATA_ROW_ID, data_row),
                (CELL_A_ID, cell_a),
                (CELL_B_ID, cell_b),
            ],
            tree: Some(Tree::new(TABLE_ID)),
            tree_id: TreeId::ROOT,
            focus: CELL_A_ID,
        };
        let tree = crate::tree::Tree::new(update, false);

        let state = tree.state();

        let col_a_node = state.node_by_id(nid(COL_A_ID)).unwrap();
        assert_eq!(col_a_node.column_index(), Some(0));

        let col_b_node = state.node_by_id(nid(COL_B_ID)).unwrap();
        assert_eq!(col_b_node.column_index(), Some(1));

        let cell_a_node = state.node_by_id(nid(CELL_A_ID)).unwrap();
        assert_eq!(cell_a_node.column_index(), Some(0));
        assert_eq!(cell_a_node.row_index(), Some(1));

        let cell_b_node = state.node_by_id(nid(CELL_B_ID)).unwrap();
        assert_eq!(cell_b_node.column_index(), Some(1));
        assert_eq!(cell_b_node.row_index(), Some(1));
    }

    #[test]
    fn table_scroll_and_action_round_trip() {
        use accesskit::Action;
        const TABLE_ID: LocalNodeId = LocalNodeId(200);
        const HEADER_ROW_ID: LocalNodeId = LocalNodeId(201);
        const HEADER_ID: LocalNodeId = LocalNodeId(202);
        const ROW_ID: LocalNodeId = LocalNodeId(203);
        const CELL_ID: LocalNodeId = LocalNodeId(204);

        // Build a table with scroll position and actions, matching
        // what the dispel-gui TableWidget generates.
        let mut table = Node::new(Role::Table);
        table.set_row_count(100);
        table.set_column_count(3);
        table.set_scroll_y(42.0);
        table.set_scroll_x(10.0);
        table.set_children(vec![HEADER_ROW_ID, ROW_ID]);

        let mut header_row = Node::new(Role::Row);
        header_row.set_row_index(0);
        header_row.set_children(vec![HEADER_ID]);

        let mut header = Node::new(Role::ColumnHeader);
        header.set_column_index(2);
        header.set_label("Header Name");

        let mut row = Node::new(Role::Row);
        row.set_row_index(5);
        row.add_action(Action::Focus);
        row.add_action(Action::ScrollIntoView);
        row.set_children(vec![CELL_ID]);

        let mut cell = Node::new(Role::Cell);
        cell.set_column_index(2);
        cell.set_row_index(5);
        cell.set_column_span(1);
        cell.set_row_span(1);
        cell.set_value("test_value");
        cell.add_action(Action::Focus);
        cell.add_action(Action::ScrollIntoView);
        cell.push_labelled_by(HEADER_ID);

        let update = TreeUpdate {
            nodes: vec![
                (TABLE_ID, table),
                (HEADER_ROW_ID, header_row),
                (HEADER_ID, header),
                (ROW_ID, row),
                (CELL_ID, cell),
            ],
            tree: Some(Tree::new(TABLE_ID)),
            tree_id: TreeId::ROOT,
            focus: TABLE_ID,
        };
        let tree = crate::tree::Tree::new(update, false);
        let state = tree.state();

        // Table properties
        let table_node = state.node_by_id(nid(TABLE_ID)).unwrap();
        assert_eq!(table_node.data().scroll_y(), Some(42.0));
        assert_eq!(table_node.data().scroll_x(), Some(10.0));
        assert_eq!(table_node.data().row_count(), Some(100));
        assert_eq!(table_node.data().column_count(), Some(3));

        // Row properties
        let row_node = state.node_by_id(nid(ROW_ID)).unwrap();
        assert_eq!(row_node.row_index(), Some(5));
        assert!(row_node.data().supports_action(Action::Focus));
        assert!(row_node.data().supports_action(Action::ScrollIntoView));

        // Cell properties
        let cell_node = state.node_by_id(nid(CELL_ID)).unwrap();
        assert_eq!(cell_node.column_index(), Some(2));
        assert_eq!(cell_node.row_index(), Some(5));
        assert_eq!(cell_node.column_span(), Some(1));
        assert_eq!(cell_node.row_span(), Some(1));
        assert_eq!(cell_node.value(), Some("test_value".into()));
        assert!(cell_node.data().supports_action(Action::Focus));
        assert!(cell_node.data().supports_action(Action::ScrollIntoView));

        // labelled_by should point to the header
        let labelled: alloc::vec::Vec<_> = cell_node.labelled_by().collect();
        assert_eq!(labelled.len(), 1);
        assert_eq!(labelled[0].id(), nid(HEADER_ID));

        // Header properties
        let header_node = state.node_by_id(nid(HEADER_ID)).unwrap();
        assert_eq!(header_node.column_index(), Some(2));
        assert_eq!(header_node.data().label(), Some("Header Name".into()));
    }
}
