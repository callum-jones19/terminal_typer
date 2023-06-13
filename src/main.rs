use std::{
    io::{self, LineWriter},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    style::Color,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lipsum::lipsum;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    text::{Line, Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Clone)]
struct GameChar {
    expected_char: char,
    given_char: Option<char>,
}

enum CharStatus {
    Correct,
    Incorrect,
    Empty,
}

#[derive(Clone)]
struct GameString {
    game_string: Vec<GameChar>,
    curr_index: usize,
}

impl GameString {
    pub fn from(s: String) -> GameString {
        let mut res = Vec::new();
        for character in s.chars() {
            res.push(GameChar {
                expected_char: character,
                given_char: None,
            });
        }

        GameString {
            game_string: res,
            curr_index: 0,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.curr_index == self.game_string.len()
    }

    pub fn percentage_correct(&self) -> f32 {
        let mut res = 0;
        for i in 0..self.curr_index {
            match self.game_string[i].given_char {
                Some(typed) => {
                    if typed == self.game_string[i].expected_char {
                        res += 1;
                    }
                }
                None => {}
            }
        }
        let divisor = self.curr_index;
        if divisor == 0 {
            0.0
        } else {
            let fraction = (res as f32) / (divisor as f32);
            (fraction * 100.0).round() as f32
        }
    }

    pub fn status_at_index(&self, index: usize) -> CharStatus {
        if index >= self.game_string.len() {
            CharStatus::Empty
        } else {
            match self.game_string[index].given_char {
                Some(given) => {
                    if given == self.game_string[index].expected_char {
                        CharStatus::Correct
                    } else {
                        CharStatus::Incorrect
                    }
                }
                None => CharStatus::Empty,
            }
        }
    }

    pub fn update_next_char(&mut self, new_char: char) {
        if self.curr_index < self.game_string.len() {
            self.game_string[self.curr_index].given_char = Some(new_char);
            self.curr_index += 1;
        }
    }

    pub fn pop_char(&mut self) {
        if self.curr_index > 0 {
            self.curr_index -= 1;
            self.game_string[self.curr_index].given_char = None
        }
    }
}

#[derive(Clone)]
struct Round {
    text: GameString,
    start_time: Instant,
    end_time: Option<Instant>,
}

impl Round {
    pub fn new(target_str: String) -> Self {
        Round {
            text: GameString::from(target_str),
            start_time: Instant::now(),
            end_time: None,
        }
    }

    pub fn calculate_wpm(&self) -> i32 {
        90
    }

    pub fn is_complete(&self) -> bool {
        match self.end_time {
            Some(_) => true,
            None => false,
        }
    }

    pub fn handle_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char(typed) => {
                self.text.update_next_char(typed);
                if self.text.is_completed() {
                    self.end_time = Some(Instant::now());
                }
            }
            KeyCode::Backspace => {
                self.text.pop_char();
            }
            _ => {}
        }
    }
}

enum GameStatus {
    Waiting,
    Ongoing(Round),
    Complete,
}

struct Game {
    status: GameStatus,
    record: Vec<Round>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            status: GameStatus::Waiting,
            record: Vec::new(),
        }
    }

    pub fn elapsed_time(&self) -> Duration {
        match &self.status {
            GameStatus::Waiting => Duration::ZERO,
            GameStatus::Ongoing(round) => round.start_time.elapsed(),
            GameStatus::Complete => Duration::ZERO,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        // Check for exit
        match key.code {
            KeyCode::Esc => return true,
            _ => {}
        }

        // Handle controls through the round state.
        let mut finished_round = None;
        match &mut self.status {
            GameStatus::Waiting => {
                // Enter the letter given and start the game
                match key.code {
                    KeyCode::Enter => {
                        self.status = GameStatus::Ongoing(Round::new(lipsum(5)));
                    }
                    _ => {}
                }
            }
            GameStatus::Ongoing(round) => {
                round.handle_input(&key);
                if round.end_time.is_some() {
                    finished_round = Some(round.clone());
                }
            }
            GameStatus::Complete => match key.code {
                KeyCode::Enter => {
                    self.status = GameStatus::Waiting;
                }
                _ => {}
            },
        }

        // Update GameState if necessary
        match finished_round {
            Some(round) => {
                self.status = GameStatus::Complete;
                self.record.push(round);
            }
            None => {}
        }

        false
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, game: &mut Game) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    // let prompt_text = Span::raw(&game.target_phrase);
    match &game.status {
        GameStatus::Waiting => {
            let waiting_msg = " 🏎️🏎️🏎️🏎️🏎️🏎️🏎️🏎️🏎️  ";
            let title_box = Paragraph::new(Spans::from(waiting_msg))
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
        GameStatus::Ongoing(round) => {
            let mut rendered_text = Vec::new();
            for i in 0..round.text.game_string.len() {
                let style = match round.text.status_at_index(i) {
                    CharStatus::Correct => Style::default().fg(ratatui::style::Color::Green),
                    CharStatus::Incorrect => Style::default().fg(ratatui::style::Color::Red),
                    CharStatus::Empty => Style::default().fg(ratatui::style::Color::Gray),
                };

                // TODO abstract this functionality into the class. Fine here for now.
                let rendered_char = match round.text.game_string[i].given_char {
                    Some(c) => {
                        if c != ' ' {
                            c
                        } else {
                            '·'
                        }
                    }
                    None => round.text.game_string[i].expected_char,
                };
                rendered_text.push(Span::styled(rendered_char.to_string(), style));
            }

            let prompt_box = Paragraph::new(Spans::from(rendered_text))
                .block(Block::default().title(" Prompt ").borders(Borders::ALL))
                .style(
                    Style::default()
                        .fg(ratatui::style::Color::White)
                        .bg(ratatui::style::Color::Black),
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);
            f.render_widget(prompt_box, chunks[0]);

            let accuracy = format!(
                "Word Accuracy: {}% \t \t Time Elapsed: {:?}",
                round.text.percentage_correct(),
                game.elapsed_time()
            );
            let block2 = Paragraph::new(accuracy)
                .block(Block::default().title("Stats").borders(Borders::ALL))
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
                .block(
                    Block::default()
                        .title(" Leaderboard ")
                        .borders(Borders::ALL),
                )
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

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // let target_str = String::from("This is the string we have to try to type. Good luck!");
    let mut game = Game::new();

    loop {
        terminal.draw(|f| {
            ui(f, &mut game);
        })?;

        if let Event::Key(key) = event::read()? {
            if game.handle_input(key) == true {
                break;
            }
        };
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
