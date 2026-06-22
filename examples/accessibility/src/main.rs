//! An example showcasing accessibility labels and properties across all
//! accessible Iced widgets.
//!
//! This example demonstrates how to provide accessible labels, tooltip
//! notifications, table roles, pane labels, and other properties that screen
//! readers and assistive technologies rely on.
//!
//! Run it with:
//! ```bash
//! cargo run -p accessibility
//! ```
use iced::widget::pane_grid;
use iced::widget::{
    button, checkbox, column, container, pane_grid as pg, progress_bar, qr_code, radio, row,
    scrollable, slider, table, text, text_input, toggler, tooltip, vertical_slider,
};
use iced::{Center, Element, Fill, Length};

/// Runs the accessibility demo application.
pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Accessibility Demo")
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    CheckboxToggled(bool),
    TogglerToggled(bool),
    RadioSelected(i32),
    SliderChanged(u8),
    VSliderChanged(i32),
    TextInputChanged(String),
    CounterIncrement,
    CounterDecrement,
}

struct Person {
    name: String,
}

struct App {
    checkbox: bool,
    toggler: bool,
    radio: i32,
    slider: u8,
    vslider: i32,
    text_input: String,
    counter: i64,
    panes: pane_grid::State<u32>,
    people: Vec<Person>,
    qr_data: Option<qr_code::Data>,
}

impl App {
    fn new() -> Self {
        let (panes, _) = pane_grid::State::new(0);
        Self {
            checkbox: false,
            toggler: false,
            radio: 0,
            slider: 50,
            vslider: 0,
            text_input: String::new(),
            counter: 0,
            panes,
            people: vec![
                Person {
                    name: "Alice".into(),
                },
                Person { name: "Bob".into() },
                Person {
                    name: "Charlie".into(),
                },
            ],
            qr_data: qr_code::Data::new("https://iced.rs").ok(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CheckboxToggled(v) => self.checkbox = v,
            Message::TogglerToggled(v) => self.toggler = v,
            Message::RadioSelected(v) => self.radio = v,
            Message::SliderChanged(v) => self.slider = v,
            Message::VSliderChanged(v) => self.vslider = v,
            Message::TextInputChanged(v) => self.text_input = v,
            Message::CounterIncrement => self.counter += 1,
            Message::CounterDecrement => self.counter -= 1,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            self.heading_section(),
            self.interactive_section(),
            self.slider_section(),
            self.input_section(),
            self.data_section(),
            self.layout_section(),
        ]
        .spacing(24)
        .padding(20);

        scrollable(content).height(Fill).into()
    }

    fn heading_section(&self) -> Element<'_, Message> {
        column![
            text("Accessibility Demo").heading(1).size(32),
            text(
                "This application demonstrates accessible labels \
                 and properties across Iced widgets."
            )
            .heading(3),
            text("Use a screen reader to explore the labeled controls below."),
        ]
        .spacing(8)
        .into()
    }

    fn interactive_section(&self) -> Element<'_, Message> {
        column![
            text("Interactive Controls").heading(2),
            row![
                tooltip(
                    button("Increment").on_press(Message::CounterIncrement),
                    text("Adds 1 to the counter"),
                    tooltip::Position::Top,
                ),
                tooltip(
                    button("Decrement").on_press(Message::CounterDecrement),
                    text("Subtracts 1 from the counter"),
                    tooltip::Position::Top,
                ),
                text(self.counter).size(24),
            ]
            .spacing(10)
            .align_y(Center),
            row![
                checkbox(self.checkbox)
                    .label("Enable notifications")
                    .on_toggle(Message::CheckboxToggled),
                toggler(self.toggler)
                    .label("Dark mode")
                    .on_toggle(Message::TogglerToggled),
            ]
            .spacing(20)
            .align_y(Center),
            row![
                radio("Option A", 0, Some(self.radio), Message::RadioSelected),
                radio("Option B", 1, Some(self.radio), Message::RadioSelected),
                radio("Option C", 2, Some(self.radio), Message::RadioSelected),
            ]
            .spacing(16),
        ]
        .spacing(12)
        .into()
    }

    fn slider_section(&self) -> Element<'_, Message> {
        column![
            text("Sliders").heading(2),
            slider(0..=100, self.slider, Message::SliderChanged).accessible_label("Volume"),
            vertical_slider(0..=100, self.vslider, Message::VSliderChanged)
                .accessible_label("Brightness"),
        ]
        .spacing(12)
        .width(Length::FillPortion(2))
        .into()
    }

    fn input_section(&self) -> Element<'_, Message> {
        column![
            text("Text & Progress").heading(2),
            text_input("Type here...", &self.text_input)
                .on_input(Message::TextInputChanged)
                .accessible_label("Search query"),
            progress_bar(0.0..=100.0, self.slider as f32).accessible_label("Volume progress"),
        ]
        .spacing(12)
        .into()
    }

    fn data_section(&self) -> Element<'_, Message> {
        column![
            text("Data Widgets").heading(2),
            row![
                column![
                    text("QR Code"),
                    match &self.qr_data {
                        Some(data) => {
                            let qr: Element<'_, Message> = iced::widget::qr_code(data)
                                .accessible_label("QR Code for Iced website")
                                .encoded_value("https://iced.rs")
                                .into();
                            qr
                        }
                        None => text("(QR generation failed)").into(),
                    },
                ]
                .spacing(4),
            ]
            .spacing(20),
            table::table(
                [table::column(text("Name"), |person: &Person| {
                    text(&person.name)
                })],
                &self.people[..],
            ),
        ]
        .spacing(12)
        .into()
    }

    fn layout_section(&self) -> Element<'_, Message> {
        let pane_grid = pg::PaneGrid::new(&self.panes, |_id, _pane, _maximized| {
            let body: Element<'_, Message> =
                container(text("Pane with accessible label").heading(3))
                    .center_x(Fill)
                    .center_y(Fill)
                    .into();

            pg::Content::new(body).accessible_label("Accessible Pane")
        })
        .width(Fill)
        .height(Length::Fixed(200.0));

        column![text("Layout Widgets").heading(2), pane_grid,]
            .spacing(12)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::Role;
    use iced_test::simulator;

    #[test]
    fn buttons_dispatch_messages() {
        let app = App::new();
        let mut ui = simulator(app.view());
        let tree = ui.accessibility_tree();
        let buttons: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Button)
            .collect();
        // There are at least 2 buttons (Increment, Decrement)
        assert!(buttons.len() >= 2);
        assert!(buttons[0].1.supports_action(accesskit::Action::Click));
    }

    #[test]
    fn sliders_have_accessible_labels() {
        let app = App::new();
        let mut ui = simulator(app.view());
        let tree = ui.accessibility_tree();
        let sliders: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Slider)
            .collect();
        assert_eq!(sliders.len(), 2, "Should have horizontal + vertical slider");
        assert_eq!(sliders[0].1.label(), Some("Volume"));
    }

    #[test]
    fn table_exists_with_header_and_rows() {
        let app = App::new();
        let mut ui = simulator(app.view());
        let tree = ui.accessibility_tree();
        let table_node = tree
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Table)
            .map(|(_, n)| n);
        assert!(table_node.is_some(), "Table node should exist");
        // 1 header + 3 data rows
        assert_eq!(table_node.unwrap().row_count(), Some(4));
    }

    #[test]
    fn qr_code_has_custom_label_and_value() {
        let app = App::new();
        let mut ui = simulator(app.view());
        let tree = ui.accessibility_tree();
        let qr_node = tree
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Image && n.label() == Some("QR Code for Iced website"))
            .map(|(_, n)| n);
        assert!(qr_node.is_some(), "QR Code with custom label should exist");
        assert_eq!(qr_node.unwrap().value(), Some("https://iced.rs"));
    }
}
