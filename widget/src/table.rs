//! Display tables.
use crate::core;
use crate::core::alignment;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    Alignment, Background, Element, Layout, Length, Pixels, Rectangle, Size, Widget,
};

/// Creates a new [`Table`] with the given columns and rows.
///
/// Columns can be created using the [`column()`] function, while rows can be any
/// iterator over some data type `T`.
pub fn table<'a, 'b, T, Message, Theme, Renderer>(
    columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
    rows: impl IntoIterator<Item = T>,
) -> Table<'a, Message, Theme, Renderer>
where
    T: Clone,
    Theme: Catalog,
    Renderer: core::Renderer,
{
    Table::new(columns, rows)
}

/// Creates a new [`Column`] with the given header and view function.
///
/// The view function will be called for each row in a [`Table`] and it must
/// produce the resulting contents of a cell.
pub fn column<'a, 'b, T, E, Message, Theme, Renderer>(
    header: impl Into<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(T) -> E + 'b,
) -> Column<'a, 'b, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    Column {
        header: header.into(),
        view: Box::new(move |data| view(data).into()),
        width: Length::Shrink,
        align_x: alignment::Horizontal::Left,
        align_y: alignment::Vertical::Top,
        row_header: false,
        sort_direction: None,
    }
}

/// The direction used to sort a table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Values are sorted in ascending order.
    Ascending,
    /// Values are sorted in descending order.
    Descending,
    /// Values use an application-defined ordering.
    Other,
}

/// A grid-like visual representation of data distributed in columns and rows.
pub struct Table<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
{
    columns: Vec<Column_>,
    cells: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    padding_x: f32,
    padding_y: f32,
    separator_x: f32,
    separator_y: f32,
    accessible_label: Option<String>,
    selected_rows: Option<Vec<usize>>,
    selected_cells: Option<Vec<(usize, usize)>>,
    selected_columns: Option<Vec<usize>>,
    class: Theme::Class<'a>,
}

struct Column_ {
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    #[cfg_attr(not(feature = "accessibility"), allow(dead_code))]
    row_header: bool,
    #[cfg_attr(not(feature = "accessibility"), allow(dead_code))]
    sort_direction: Option<SortDirection>,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    /// Creates a new [`Table`] with the given columns and rows.
    ///
    /// Columns can be created using the [`column()`] function, while rows can be any
    /// iterator over some data type `T`.
    pub fn new<'b, T>(
        columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
        rows: impl IntoIterator<Item = T>,
    ) -> Self
    where
        T: Clone,
    {
        let columns = columns.into_iter();
        let rows = rows.into_iter();

        let mut width = Length::Fit;
        let mut cells = Vec::with_capacity(columns.size_hint().0 * (1 + rows.size_hint().0));

        let (mut columns, views): (Vec<_>, Vec<_>) = columns
            .map(|column| {
                width = width.stack(column.width);

                cells.push(column.header);

                (
                    Column_ {
                        width: column.width,
                        align_x: column.align_x,
                        align_y: column.align_y,
                        row_header: column.row_header,
                        sort_direction: column.sort_direction,
                    },
                    column.view,
                )
            })
            .collect();

        if width == Length::Shrink
            && let Some(first) = columns.first_mut()
        {
            first.width = Length::Fill;
        }

        for row in rows {
            for view in &views {
                let cell = view(row.clone());
                cells.push(cell);
            }
        }

        Self {
            columns,
            cells,
            width,
            height: Length::Fit,
            padding_x: 10.0,
            padding_y: 5.0,
            separator_x: 1.0,
            separator_y: 1.0,
            accessible_label: None,
            selected_rows: None,
            selected_cells: None,
            selected_columns: None,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`Table`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the padding of the cells of the [`Table`].
    pub fn padding(self, padding: impl Into<Pixels>) -> Self {
        let padding = padding.into();

        self.padding_x(padding).padding_y(padding)
    }

    /// Sets the horizontal padding of the cells of the [`Table`].
    pub fn padding_x(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_x = padding.into().0;
        self
    }

    /// Sets the vertical padding of the cells of the [`Table`].
    pub fn padding_y(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_y = padding.into().0;
        self
    }

    /// Sets the thickness of the line separator between the cells of the [`Table`].
    pub fn separator(self, separator: impl Into<Pixels>) -> Self {
        let separator = separator.into();

        self.separator_x(separator).separator_y(separator)
    }

    /// Sets the thickness of the horizontal line separator between the cells of the [`Table`].
    pub fn separator_x(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_x = separator.into().0;
        self
    }

    /// Sets the thickness of the vertical line separator between the cells of the [`Table`].
    pub fn separator_y(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_y = separator.into().0;
        self
    }

    /// Sets the accessible label of the [`Table`].
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Marks data rows as selectable and identifies the selected rows.
    ///
    /// Row indices are zero-based and do not include the header row.
    pub fn selected_rows(mut self, rows: impl IntoIterator<Item = usize>) -> Self {
        self.selected_rows = Some(rows.into_iter().collect());
        self
    }

    /// Marks data cells as selectable and identifies the selected cells.
    ///
    /// Each `(row, column)` index is zero-based and row indices do not include
    /// the header row.
    pub fn selected_cells(mut self, cells: impl IntoIterator<Item = (usize, usize)>) -> Self {
        self.selected_cells = Some(cells.into_iter().collect());
        self
    }

    /// Marks columns as selectable and identifies the selected columns.
    ///
    /// Column indices are zero-based.
    pub fn selected_columns(mut self, columns: impl IntoIterator<Item = usize>) -> Self {
        self.selected_columns = Some(columns.into_iter().collect());
        self
    }
}

struct Metrics {
    columns: Vec<f32>,
    rows: Vec<f32>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Metrics>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Metrics {
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    fn diff(&mut self, tree: &mut widget::Tree) {
        tree.diff_children(&mut self.cells);

        for cell in &self.cells {
            let size = cell.as_widget().size();

            self.height = self.height.stack(size.height);
        }
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let metrics = tree.state.downcast_mut::<Metrics>();
        let columns = self.columns.len();
        let rows = self.cells.len() / columns;

        let limits = limits.width(self.width).height(self.height);
        let available = limits.max();
        let table_fluid = if self.width.fill_factor() == 0 {
            Length::Shrink
        } else {
            Length::Fill
        };

        let mut cells = Vec::with_capacity(self.cells.len());
        cells.resize(self.cells.len(), layout::Node::default());

        metrics.columns = vec![0.0; self.columns.len()];
        metrics.rows = vec![0.0; rows];

        let mut column_factors = vec![0; self.columns.len()];
        let mut total_row_factors = 0;
        let mut total_fluid_height = 0.0;
        let mut row_factor = 0;

        let spacing_x = self.padding_x * 2.0 + self.separator_x;
        let spacing_y = self.padding_y * 2.0 + self.separator_y;

        // FIRST PASS
        // Lay out non-fluid cells
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in self.cells.iter_mut().zip(&mut tree.children).enumerate() {
            let row = i / columns;
            let column = i % columns;

            let width = self.columns[column].width;
            let size = cell.as_widget().size();

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;

                    if row_factor != 0 {
                        total_fluid_height += metrics.rows[row - 1];
                        total_row_factors += row_factor;

                        row_factor = 0;
                    }
                }
            }

            let width_factor = width.fill_factor();
            let height_factor = size.height.fill_factor();

            if width_factor != 0 || height_factor != 0 || size.width.is_fill() {
                column_factors[column] = column_factors[column].max(width_factor);

                row_factor = row_factor.max(height_factor);

                continue;
            }

            let limits = layout::Limits::new(
                Size::ZERO,
                Size::new(available.width - x, available.height - y),
            )
            .width(width);

            let layout = cell.as_widget_mut().layout(state, renderer, &limits);
            let size = limits.resolve(width, Length::Shrink, layout.size());

            metrics.columns[column] = metrics.columns[column].max(size.width);
            metrics.rows[row] = metrics.rows[row].max(size.height);
            cells[i] = layout;

            x += size.width + spacing_x;
        }

        // SECOND PASS
        // Lay out fluid cells, using metrics from the first pass as limits
        let left = Size::new(
            available.width
                - metrics
                    .columns
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| column_factors[*i] == 0)
                    .map(|(_, width)| width)
                    .sum::<f32>(),
            available.height - total_fluid_height,
        );

        let width_unit = (left.width
            - spacing_x * self.columns.len().saturating_sub(1) as f32
            - self.padding_x * 2.0)
            / column_factors.iter().sum::<u16>() as f32;

        let height_unit =
            (left.height - spacing_y * rows.saturating_sub(1) as f32 - self.padding_y * 2.0)
                / total_row_factors as f32;

        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in self.cells.iter_mut().zip(&mut tree.children).enumerate() {
            let row = i / columns;
            let column = i % columns;

            let size = cell.as_widget().size();

            let width = self.columns[column].width;
            let width_factor = width.fill_factor();
            let height_factor = size.height.fill_factor();

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            if width_factor == 0 && size.width.fill_factor() == 0 && size.height.fill_factor() == 0
            {
                continue;
            }

            let max_width = if width_factor == 0 {
                if size.width.is_fill() {
                    metrics.columns[column]
                } else {
                    (available.width - x).max(0.0)
                }
            } else {
                width_unit * width_factor as f32
            };

            let max_height = if height_factor == 0 {
                if size.height.is_fill() {
                    metrics.rows[row]
                } else {
                    (available.height - y).max(0.0)
                }
            } else {
                height_unit * height_factor as f32
            };

            let limits =
                layout::Limits::new(Size::ZERO, Size::new(max_width, max_height)).width(width);

            let layout = cell.as_widget_mut().layout(state, renderer, &limits);
            let size = limits.resolve(
                if let Length::Fixed(_) = width {
                    width
                } else {
                    table_fluid
                },
                Length::Shrink,
                layout.size(),
            );

            metrics.columns[column] = metrics.columns[column].max(size.width);
            metrics.rows[row] = metrics.rows[row].max(size.height);
            cells[i] = layout;

            x += size.width + spacing_x;
        }

        // THIRD PASS
        // Position each cell
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, cell) in cells.iter_mut().enumerate() {
            let row = i / columns;
            let column = i % columns;

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            let Column_ {
                align_x, align_y, ..
            } = &self.columns[column];

            cell.move_to_mut((x, y));
            cell.align_mut(
                Alignment::from(*align_x),
                Alignment::from(*align_y),
                Size::new(metrics.columns[column], metrics.rows[row]),
            );

            x += metrics.columns[column] + spacing_x;
        }

        let intrinsic = limits.resolve(
            self.width,
            self.height,
            Size::new(
                x - spacing_x + self.padding_x,
                y + metrics
                    .rows
                    .last()
                    .copied()
                    .map(|height| height + self.padding_y)
                    .unwrap_or_default(),
            ),
        );

        layout::Node::with_children(intrinsic, cells)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &core::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((cell, tree), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut()
                .update(tree, event, layout, cursor, renderer, shell, viewport);
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((cell, state), layout) in self.cells.iter().zip(&tree.children).zip(layout.children())
        {
            cell.as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }

        let bounds = layout.bounds();
        let metrics = tree.state.downcast_ref::<Metrics>();
        let style = theme.style(&self.class);

        if self.separator_x > 0.0 {
            let mut x = self.padding_x;

            for width in &metrics.columns[..metrics.columns.len().saturating_sub(1)] {
                x += width + self.padding_x;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + x,
                            y: bounds.y,
                            width: self.separator_x,
                            height: bounds.height,
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    style.separator_x,
                );

                x += self.separator_x + self.padding_x;
            }
        }

        if self.separator_y > 0.0 {
            let mut y = self.padding_y;

            for height in &metrics.rows[..metrics.rows.len().saturating_sub(1)] {
                y += height + self.padding_y;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y + y,
                            width: bounds.width,
                            height: self.separator_y,
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    style.separator_y,
                );

                y += self.separator_y + self.padding_y;
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((cell, tree), layout)| {
                cell.as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        for ((cell, state), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut()
                .operate(state, layout, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: core::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.cells,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }

    #[cfg(feature = "accessibility")]
    fn accessibility(
        &self,
        layout: crate::core::Layout<'_>,
        tree: &crate::core::widget::Tree,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        id_counter: &mut u64,
    ) -> Option<accesskit::NodeId> {
        use accesskit::Role;

        let columns = self.columns.len();
        let grid_rows = if columns > 0 {
            self.cells.len() / columns
        } else {
            0
        };
        let rows = grid_rows.saturating_sub(1);

        let mut header_ids: Vec<accesskit::NodeId> = Vec::new();
        let mut header_bounds: Option<crate::core::Rectangle> = None;

        // Get child node IDs, grouping data cells by row. Headers are kept
        // separate because native table APIs do not count them as data rows.
        let mut row_cell_ids: Vec<Vec<accesskit::NodeId>> = Vec::new();
        let mut row_bounds: Vec<Option<crate::core::Rectangle>> = Vec::new();

        for (i, (cell, child_tree)) in self.cells.iter().zip(tree.children.iter()).enumerate() {
            let col = i % columns;
            let row = i / columns;

            let is_header = row == 0;
            let data_row = row.saturating_sub(1);

            if !is_header && col == 0 {
                row_cell_ids.push(Vec::new());
                row_bounds.push(None);
            }

            let cell_layout = layout.children().nth(i);
            if let Some(cell_layout) = cell_layout {
                if let Some(cell_id) =
                    cell.as_widget()
                        .accessibility(cell_layout, child_tree, nodes, id_counter)
                {
                    // Wrap the content in a semantic header or data cell.
                    let wrapper_id = accesskit::NodeId(*id_counter);
                    *id_counter += 1;

                    let mut wrapper = accesskit::Node::new(if is_header {
                        Role::ColumnHeader
                    } else if self.columns[col].row_header {
                        Role::RowHeader
                    } else {
                        Role::Cell
                    });
                    wrapper.push_child(cell_id);
                    let cell_bounds = nodes
                        .iter()
                        .find(|(id, _)| *id == cell_id)
                        .and_then(|(_, child)| child.bounds())
                        .map(crate::core::accessibility::from_rect)
                        .unwrap_or_else(|| cell_layout.bounds());
                    let cell_bounds = crate::core::accessibility::non_empty(cell_bounds);

                    if let Some((_, child)) = nodes.iter().find(|(id, _)| *id == cell_id) {
                        if let Some(label) = child.label().or_else(|| child.value()) {
                            wrapper.set_label(label);
                        }
                    }
                    wrapper.set_bounds(crate::core::accessibility::rect(cell_bounds));
                    wrapper.set_column_index(col);
                    wrapper.set_row_span(1);
                    wrapper.set_column_span(1);
                    if is_header {
                        if let Some(sort_direction) = self.columns[col].sort_direction {
                            wrapper.set_sort_direction(match sort_direction {
                                SortDirection::Ascending => accesskit::SortDirection::Ascending,
                                SortDirection::Descending => accesskit::SortDirection::Descending,
                                SortDirection::Other => accesskit::SortDirection::Other,
                            });
                        }
                        if let Some(selected_columns) = &self.selected_columns {
                            wrapper.set_selected(selected_columns.contains(&col));
                        }
                    } else {
                        wrapper.set_row_index(data_row);
                        if let Some(selected_cells) = &self.selected_cells {
                            wrapper.set_selected(selected_cells.contains(&(data_row, col)));
                        }
                    }
                    nodes.push((wrapper_id, wrapper));

                    if is_header {
                        header_ids.push(wrapper_id);
                    } else if let Some(last_row) = row_cell_ids.last_mut() {
                        last_row.push(wrapper_id);
                    }

                    if is_header {
                        header_bounds = Some(header_bounds.map_or(cell_bounds, |bounds| {
                            crate::core::accessibility::union(bounds, cell_bounds)
                        }));
                    } else if let Some(bounds) = row_bounds.get_mut(data_row) {
                        *bounds = Some(bounds.map_or(cell_bounds, |row_bounds| {
                            crate::core::accessibility::union(row_bounds, cell_bounds)
                        }));
                    }
                }
            }
        }

        // Create Row wrapper nodes for each row
        let mut row_ids: Vec<accesskit::NodeId> = Vec::new();

        for (row, row_cells) in row_cell_ids.iter().enumerate() {
            let row_id = accesskit::NodeId(*id_counter);
            *id_counter += 1;

            let mut row_builder = accesskit::Node::new(Role::Row);
            if let Some(Some(bounds)) = row_bounds.get(row) {
                crate::core::accessibility::set_bounds(tree, &mut row_builder, *bounds);
            }
            row_builder.set_row_index(row);
            row_builder.set_column_index(0);
            row_builder.set_column_span(columns);
            if let Some(selected_rows) = &self.selected_rows {
                row_builder.set_selected(selected_rows.contains(&row));
            }

            for cell_id in row_cells {
                row_builder.push_child(*cell_id);
            }
            nodes.push((row_id, row_builder));
            row_ids.push(row_id);
        }

        let header_group_id = (!header_ids.is_empty()).then(|| {
            let header_group_id = accesskit::NodeId(*id_counter);
            *id_counter += 1;

            let mut header_group = accesskit::Node::new(Role::RowGroup);
            if let Some(bounds) = header_bounds {
                crate::core::accessibility::set_bounds(tree, &mut header_group, bounds);
            }
            header_group.set_children(header_ids);
            nodes.push((header_group_id, header_group));

            header_group_id
        });

        // Create the Table node
        let table_id = accesskit::NodeId(*id_counter);
        *id_counter += 1;

        let mut table_builder = accesskit::Node::new(Role::Table);
        crate::core::accessibility::set_bounds(tree, &mut table_builder, layout.bounds());
        if let Some(label) = &self.accessible_label {
            table_builder.set_label(label.as_str());
        }
        if let Some(header_group_id) = header_group_id {
            table_builder.push_child(header_group_id);
        }
        for row_id in &row_ids {
            table_builder.push_child(*row_id);
        }
        table_builder.set_row_count(rows);
        table_builder.set_column_count(columns);
        if self
            .selected_rows
            .as_ref()
            .is_some_and(|rows| rows.len() > 1)
            || self
                .selected_cells
                .as_ref()
                .is_some_and(|cells| cells.len() > 1)
            || self
                .selected_columns
                .as_ref()
                .is_some_and(|columns| columns.len() > 1)
        {
            table_builder.set_multiselectable();
        }
        nodes.push((table_id, table_builder));

        Some(table_id)
    }

    #[cfg(feature = "accessibility")]
    fn accessibility_action(
        &mut self,
        tree: &mut crate::core::widget::Tree,
        layout: crate::core::Layout<'_>,
        action: &accesskit::ActionRequest,
        shell: &mut crate::core::Shell<'_, Message>,
    ) {
        for ((cell, child_tree), child_layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            if !child_tree.contains_accesskit_node_id(action.target_node) {
                continue;
            }

            cell.as_widget_mut()
                .accessibility_action(child_tree, child_layout, action, shell);
            break;
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Table<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(table: Table<'a, Message, Theme, Renderer>) -> Self {
        Element::new(table)
    }
}

/// A vertical visualization of some data with a header.
pub struct Column<'a, 'b, T, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    header: Element<'a, Message, Theme, Renderer>,
    view: Box<dyn Fn(T) -> Element<'a, Message, Theme, Renderer> + 'b>,
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    row_header: bool,
    sort_direction: Option<SortDirection>,
}

impl<'a, 'b, T, Message, Theme, Renderer> Column<'a, 'b, T, Message, Theme, Renderer> {
    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the alignment for the horizontal axis of the [`Column`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the alignment for the vertical axis of the [`Column`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.align_y = alignment.into();
        self
    }

    /// Marks the cells in this column as row headers.
    pub fn row_header(mut self, row_header: bool) -> Self {
        self.row_header = row_header;
        self
    }

    /// Describes how this column is currently sorted.
    pub fn sort_direction(mut self, direction: SortDirection) -> Self {
        self.sort_direction = Some(direction);
        self
    }
}

/// The appearance of a [`Table`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The background color of the horizontal line separator between cells.
    pub separator_x: Background,
    /// The background color of the vertical line separator between cells.
    pub separator_y: Background,
}

/// The theme catalog of a [`Table`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Table`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl<Theme> From<Style> for StyleFn<'_, Theme> {
    fn from(style: Style) -> Self {
        Box::new(move |_theme| style)
    }
}

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default style of a [`Table`].
pub fn default(theme: &crate::Theme) -> Style {
    let palette = theme.palette();
    let separator = palette.background.strong.color.into();

    Style {
        separator_x: separator,
        separator_y: separator,
    }
}
