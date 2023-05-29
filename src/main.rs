use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    style::Color,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

struct GameChar {
    expected_char: char,
    given_char: Option<char>,
}

enum CharStatus {
    Correct,
    Incorrect,
    Empty,
}

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

struct Game {
    time_limit: i32,
    curr_timer: i32,
    text: GameString,
}

impl Game {
    pub fn new(target_str: String, time_lmt: i32) -> Self {
        Game {
            time_limit: time_lmt,
            curr_timer: 0,
            text: GameString::from(target_str),
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, game: &mut Game) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    // let prompt_text = Span::raw(&game.target_phrase);
    let mut rendered_text = Vec::new();
    for i in 0..game.text.game_string.len() {
        let style = match game.text.status_at_index(i) {
            CharStatus::Correct => Style::default().fg(ratatui::style::Color::Green),
            CharStatus::Incorrect => Style::default().fg(ratatui::style::Color::Red),
            CharStatus::Empty => Style::default().fg(ratatui::style::Color::Gray),
        };
        let rendered_char = match game.text.game_string[i].given_char {
            Some(c) => c,
            None => game.text.game_string[i].expected_char,
        };
        rendered_text.push(Span::styled(rendered_char.to_string(), style));
    }
    let prompt_box = Paragraph::new(Spans::from(rendered_text))
        .block(Block::default().title("Prompt").borders(Borders::ALL))
        .style(
            Style::default()
                .fg(ratatui::style::Color::White)
                .bg(ratatui::style::Color::Black),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(prompt_box, chunks[0]);

    let block2 = Block::default().title("Stats").borders(Borders::ALL);
    f.render_widget(block2, chunks[1]);
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let target_str = String::from("This is the string we have to try to type. Good luck!");
    let mut game = Game::new(target_str, 100);

    loop {
        terminal.draw(|f| {
            ui(f, &mut game);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Char(typed) => {
                    game.text.update_next_char(typed);
                }
                KeyCode::Backspace => {
                    game.text.pop_char();
                }
                _ => {}
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
