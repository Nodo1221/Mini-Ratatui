use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Layout},
    prelude::Stylize,
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListState, Paragraph},
};

fn main() -> std::io::Result<()> {
    // Basic Block (usually useless on its own - typically embedded via .block())
    // Add title, borders, style
    let block_widget = Block::default()
        .title(Line::from("Left title"))
        .title(Line::from("Right title").alignment(Alignment::Right))
        .borders(Borders::TOP | Borders::BOTTOM);

    // List widget with a fancy styled bottom title
    let list_widget = List::new(["Item 1", "Item 2", "Item 3"])
        .block(Block::bordered()
            .title("List")
            .title_bottom(
                Line::from_iter([" Resize ".into(), "<Left / Right>".blue().bold(), " Select ".into(), "<Up / Down> ".blue().bold()])
                .centered(),
            )
            .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().fg(Color::Yellow).bold())
        .highlight_symbol(">> ");

    let mut list_state = ListState::default().with_selected(Some(1));
    let mut left_pct: u16 = 50;

    ratatui::run(|terminal| loop {
        // Paragraph block: text wrapping, scroll, alignment, style, attach a block (border, title)
        // Has to be rebuilt in the loop, since its content is dynamic
        let para_widget =
            Paragraph::new(format!("This is a paragraph widget\n\nWidth: {}%", left_pct))
                .block(Block::bordered()
                    .title("Paragraph widget")
                    .border_type(BorderType::Double),
                )
                .alignment(Alignment::Center);

        terminal.draw(|frame| {
            // Get actual Rects; generated dynamically every frame
            let [top, bottom] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Fill(1)
            ])
            .areas(frame.area());

            let [b_left, b_right] = Layout::horizontal([
                Constraint::Percentage(left_pct),
                Constraint::Percentage(100 - left_pct),
            ])
            .areas(bottom);

            frame.render_widget(&block_widget, top);
            frame.render_widget(para_widget, b_left);
            frame.render_stateful_widget(&list_widget, b_right, &mut list_state);
        })?;

        // Event handling
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Up => list_state.select_previous(),
                KeyCode::Down => list_state.select_next(),
                KeyCode::Left => left_pct = left_pct.saturating_sub(3).max(5),
                KeyCode::Right => left_pct = (left_pct + 3).min(95),
                _ => {}
            }
        }
    })
}
