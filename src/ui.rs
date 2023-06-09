use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::game::{CharStatus, Game, GameStatus};

pub fn draw<B: Backend>(f: &mut Frame<B>, game: &mut Game) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    // let prompt_text = Span::raw(&game.target_phrase);
    match &game.get_status() {
        GameStatus::Waiting => {
            // Splash screen :))
            let lines = vec![
                Line::from("________________________________    _____   .___  _______      _____   .____     "),
                Line::from("\\__    ___/\\_   _____/\\______   \\  /     \\  |   | \\      \\    /  _  \\  |    |    "),
                Line::from(".   |    |    |    __)_  |       _/ /  \\ /  \\ |   | /   |   \\  /  /_\\  \\ |    |    "),
                Line::from(".   |    |    |        \\ |    |   \\/    Y    \\|   |/    |    \\/    |    \\|    |___ "),
                Line::from(".   |____|   /_______  / |____|_  /\\____|__  /|___|\\____|__  /\\____|__  /|_______ \\"),
                Line::from(".                    \\/         \\/         \\/              \\/         \\/         \\/"),
                Line::from("________________.___.__________ _____________________                            "),
                Line::from("\\__    ___/\\__  |   |\\______   \\\\_   _____/\\______   \\                           "),
                Line::from(".   |    |    /   |   | |     ___/ |    __)_  |       _/                           "),
                Line::from(".   |    |    \\____   | |    |     |        \\ |    |   \\                           "),
                Line::from(".   |____|    / ______| |____|    /_______  / |____|_  /                           "),
                Line::from(".             \\/                          \\/         \\/                            ")
            ];

            let title_box = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" 🏎️ Terminal Typer 🏎️ ")
                        .borders(Borders::ALL),
                )
                .style(
                    Style::default()
                        .fg(ratatui::style::Color::White)
                        .bg(ratatui::style::Color::Black),
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);
            f.render_widget(title_box, chunks[0]);

            let prompt_msg = vec![
                Line::from(Span::styled(
                    "[Enter]: New Game",
                    Style::default().fg(ratatui::style::Color::Yellow),
                )),
                Line::from(Span::styled(
                    "  [Esc]: Exit Game",
                    Style::default().fg(ratatui::style::Color::Yellow),
                )),
            ];
            let prompt_box = Paragraph::new(prompt_msg)
                .block(Block::default().title(" Controls ").borders(Borders::ALL))
                .style(
                    Style::default()
                        .bg(ratatui::style::Color::Black)
                        .fg(ratatui::style::Color::White),
                );
            f.render_widget(prompt_box, chunks[1]);
        }
        GameStatus::Ongoing(round) => {
            let mut rendered_text = Vec::new();
            for i in 0..round.text.len() {
                // Assign style for character status
                let style = match round.text.status_at_index(i) {
                    CharStatus::Correct => Style::default().fg(ratatui::style::Color::Green),
                    CharStatus::Incorrect => Style::default().fg(ratatui::style::Color::Red),
                    CharStatus::Empty => Style::default().fg(ratatui::style::Color::Gray),
                };

                // TODO abstract this functionality into the class. Fine here for now.
                let rendered_char = match round.text.get_usr_given_char(i) {
                    Some(c) => {
                        if c != ' ' {
                            c
                        } else {
                            '·'
                        }
                    }
                    None => round.text.get_expected_char(i),
                };
                rendered_text.push(Span::styled(rendered_char.to_string(), style));
            }

            let prompt_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(10)
                .constraints([Constraint::Percentage(80)].as_ref())
                .split(chunks[0]);
            let prompt_block = Block::default().title(" Prompt ").borders(Borders::ALL).style(Style::default().fg(ratatui::style::Color::White).bg(ratatui::style::Color::Black));
            f.render_widget(prompt_block, chunks[0]);
            let prompt_para = Paragraph::new(Line::from(rendered_text))
                .style(
                    Style::default()
                        .fg(ratatui::style::Color::White)
                        .bg(ratatui::style::Color::Black),
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);
            f.render_widget(prompt_para, prompt_layout[0]);

            let accuracy = format!(
                "Word Accuracy: {}% \t \t Time Elapsed: {}.{}",
                round.text.percentage_correct(),
                game.elapsed_time().as_secs(),
                game.elapsed_time().subsec_millis()
            );

            let block2 = Paragraph::new(accuracy)
                .block(
                    Block::default()
                        .title("Stats")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(ratatui::style::Color::Black)),
                )
                .style(Style::default());
            f.render_widget(block2, chunks[1]);
        }
        GameStatus::Complete => {
            let waiting_msg = "Previous rounds:";
            let mut lines = vec![Line::from(Span::raw(waiting_msg))];
            for (index, round) in game.record.iter().enumerate() {
                let curr_count = index + 1;
                let new_line = Line::from(vec![
                    Span::styled("Round ", Style::default().fg(ratatui::style::Color::Green)),
                    Span::styled(
                        curr_count.to_string(),
                        Style::default().fg(ratatui::style::Color::Green),
                    ),
                    Span::raw(": "),
                    Span::raw(round.text.percentage_correct().to_string()),
                    Span::raw("% word accuracy, "),
                    Span::raw(round.calculate_wpm().to_string()),
                    Span::raw(" wpm"),
                ]);
                lines.push(new_line);
            }
            let prompt_box = Paragraph::new(lines)
                .block(Block::default().title(" Summary ").borders(Borders::ALL))
                .style(
                    Style::default()
                        .fg(ratatui::style::Color::White)
                        .bg(ratatui::style::Color::Black),
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);
            f.render_widget(prompt_box, chunks[0]);

            let prompt_msg = " [Enter]: New Game \n [Esc]: Exit Game";
            let prompt_box = Paragraph::new(prompt_msg)
                .block(Block::default().title(" Controls ").borders(Borders::ALL))
                .style(
                    Style::default()
                        .bg(ratatui::style::Color::Black)
                        .fg(ratatui::style::Color::White),
                );
            f.render_widget(prompt_box, chunks[1]);
        }
    }
}
