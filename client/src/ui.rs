use ratatui::{
    layout::Alignment,
    prelude::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let vertical = Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(50)]);
    let [command_container, information_container] = horizontal.areas(frame.size());
    let [circuit_info, boxes] = vertical.areas(information_container);

    let circuit_block = Block::default().borders(Borders::ALL).title("Circuit info");

    let mut lines: Vec<Line> = Vec::new();
    let try_lock_res = app.tor_circuits_info.try_lock();
    if try_lock_res.is_ok() {
        for circuit_info in try_lock_res.unwrap().iter() {
            lines.push(Line::from(format!(
                "Country: {:?}\n City: {:?}",
                circuit_info.country, circuit_info.city
            )))
        }
    }

    frame.render_widget(Paragraph::new(lines).block(circuit_block), circuit_info);
}
