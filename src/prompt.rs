use crate::thread_loop::ThreadLoop;
use colored::*;
use parking_lot::Mutex;
use std::fmt::Write as _;
use std::io::{stdout, Write};
use std::sync::Arc;
use termion::cursor;
use termion::raw::IntoRawMode;

pub struct Prompt {
    thread_loop: ThreadLoop,
    state: Arc<Mutex<PromptState>>,
}

pub struct PromptState {
    pub cur_offset: usize,
    pub input: String,
    pub hint: String,
}

impl Prompt {
    const PROMPT: &'static str = "> ";

    pub fn spawn() -> Self {
        let state = Arc::new(Mutex::new(PromptState::new()));

        let thread_loop = {
            let stdout = stdout();
            let mut stdout = stdout.into_raw_mode().unwrap();
            let state = Arc::clone(&state);
            ThreadLoop::spawn(50, move || {
                let state = state.lock();

                write!(
                    stdout,
                    "{}\r{}{}  {}\r{}",
                    termion::clear::CurrentLine,
                    Self::PROMPT,
                    state.input,
                    state.hint.dimmed(),
                    cursor::Right((Self::PROMPT.len() + state.input.len() - state.cur_offset) as _)
                )
                .unwrap();

                stdout.flush().unwrap();
            })
        };

        Self { thread_loop, state }
    }

    pub fn state(&self) -> Arc<Mutex<PromptState>> {
        Arc::clone(&self.state)
    }

    pub fn stop(self) {
        print!("\r\n");
        self.thread_loop.stop();
    }
}

impl PromptState {
    pub fn new() -> Self {
        Self {
            cur_offset: 0,
            input: String::with_capacity(1024),
            hint: String::with_capacity(1024),
        }
    }

    pub fn append_input(&mut self, input: &str) {
        write!(self.input, "{}", input).unwrap();
    }

    pub fn insert_input(&mut self, c: char) {
        let idx = self.input.len() - self.cur_offset;
        self.input.insert(idx, c);
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn set_hint(&mut self, hint: &str) {
        self.hint.clear();
        write!(self.hint, "{}", hint).unwrap();
    }

    pub fn cursor_left(&mut self) {
        if self.cur_offset < self.input.len() {
            self.cur_offset += 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cur_offset > 0 {
            self.cur_offset -= 1;
        }
    }

    pub fn delete_backward(&mut self) {
        let cursor_idx = self.input.len() - self.cur_offset;
        if cursor_idx > 0 {
            self.input.remove(cursor_idx - 1);
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cur_offset > 0 {
            self.input.remove(self.input.len() - self.cur_offset);
            self.cur_offset -= 1;
        }
    }
}
