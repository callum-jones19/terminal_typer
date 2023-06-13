use crossterm::event::{KeyCode, KeyEvent};
use lipsum::lipsum;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct GameChar {
    expected_char: char,
    given_char: Option<char>,
}

pub enum CharStatus {
    Correct,
    Incorrect,
    Empty,
}

#[derive(Clone)]
pub struct GameString {
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

    pub fn len(&self) -> usize {
        self.game_string.len()
    }

    pub fn is_completed(&self) -> bool {
        self.curr_index == self.game_string.len()
    }

    pub fn get_usr_given_char(&self, index: usize) -> Option<char> {
        self.game_string[index].given_char
    }

    pub fn get_expected_char(&self, index: usize) -> char {
        self.game_string[index].expected_char
    }

    pub fn words_completed(&self) -> i32 {
        // TODO more efficient if we store this data in the struct and
        // update dynamically as we go.
        let mut words = 0;
        for (index, c) in self.game_string.iter().enumerate() {
            match c.given_char {
                Some(_) => {
                    if index % 5 == 0 {
                        words += 1;
                    }
                }
                None => {
                    break;
                }
            }
        }

        words
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

// TODO encpasulate internal values
#[derive(Clone)]
pub struct Round {
    pub text: GameString,
    pub start_time: Instant,
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
        let end_time = match self.end_time {
            Some(et) => et,
            None => Instant::now(),
        };
        let time_diff = end_time.duration_since(self.start_time);
        let time_diff_mins = time_diff.as_secs_f32() / 60.0;
        let wpm = (self.text.words_completed() as f32) / time_diff_mins;

        wpm.round() as i32
    }

    // pub fn is_complete(&self) -> bool {
    //     match self.end_time {
    //         Some(_) => true,
    //         None => false,
    //     }
    // }

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

pub enum GameStatus {
    Waiting,
    Ongoing(Round),
    Complete,
}

// TODO properly encapsulate values rather than exposing the raw data types
pub struct Game {
    pub status: GameStatus,
    pub record: Vec<Round>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            status: GameStatus::Waiting,
            record: Vec::new(),
        }
    }

    pub fn get_status(&self) -> &GameStatus {
        &self.status
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
                    self.status = GameStatus::Ongoing(Round::new(lipsum(10)));
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
