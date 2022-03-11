use crate::syntax::{syntax_highlight, SyntaxToken};
use crate::thread_loop::ThreadLoop;
use colored::*;
use parking_lot::Mutex;
use std::fmt::Write as _;
use std::io::{stdout, Write};
use std::sync::Arc;
use termion::cursor;
use termion::raw::IntoRawMode;

const PROMPT: &'static str = "# ";

pub struct Prompt {
    thread_loop: ThreadLoop,
    state: Arc<Mutex<PromptState>>,
}

pub struct PromptState {
    dirty: bool,

    pub syntax_tokens: Option<Vec<SyntaxToken>>,
    pub cur_offset: usize,
    pub input: String,
    pub hint: String,
}

impl Prompt {
    pub fn spawn() -> Self {
        let state = Arc::new(Mutex::new(PromptState::new()));

        let thread_loop = {
            let stdout = stdout();
            let mut stdout = stdout.into_raw_mode().unwrap();
            let state = Arc::clone(&state);
            ThreadLoop::spawn(50, move || {
                let mut state = state.lock();
                if !state.dirty {
                    return;
                }

                write!(stdout, "{}", state).unwrap();
                stdout.flush().unwrap();
                state.mark_clean();
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
            dirty: false,
            syntax_tokens: None,
            cur_offset: 0,
            input: String::with_capacity(1024),
            hint: String::with_capacity(1024),
        }
    }

    /// Mark the state as dirty after mutating one or more fields.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    #[allow(dead_code)]
    pub fn append_input(&mut self, input: &str) {
        write!(self.input, "{}", input).unwrap();
        self.mark_dirty();
    }

    pub fn insert_input(&mut self, c: char) {
        let idx = self.input.len() - self.cur_offset;
        self.input.insert(idx, c);
        self.mark_dirty();
    }

    /// Also clears any syntax tokens and resets cursor offset.
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cur_offset = 0;
        self.clear_syntax_tokens();
        self.mark_dirty();
    }

    pub fn set_hint(&mut self, hint: &str) {
        self.hint.clear();
        write!(self.hint, "{}", hint).unwrap();
        self.mark_dirty();
    }

    pub fn clear_hint(&mut self) {
        self.hint.clear();
        self.mark_dirty();
    }

    pub fn set_syntax_tokens(&mut self, tokens: Vec<SyntaxToken>) {
        self.syntax_tokens = Some(tokens);
        self.mark_dirty();
    }

    pub fn clear_syntax_tokens(&mut self) {
        self.syntax_tokens = None;
        self.mark_dirty();
    }

    pub fn cursor_left(&mut self) {
        if self.cur_offset < self.input.len() {
            self.cur_offset += 1;
            self.mark_dirty();
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cur_offset > 0 {
            self.cur_offset -= 1;
            self.mark_dirty();
        }
    }

    pub fn delete_backward(&mut self) {
        let cursor_idx = self.input.len() - self.cur_offset;
        if cursor_idx > 0 {
            self.input.remove(cursor_idx - 1);
            self.clear_syntax_tokens();
            self.mark_dirty();
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cur_offset > 0 {
            self.input.remove(self.input.len() - self.cur_offset);
            self.cur_offset -= 1;
            self.clear_syntax_tokens();
            self.mark_dirty();
        }
    }
}

impl std::fmt::Display for PromptState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\r{}", termion::clear::CurrentLine, PROMPT.dimmed())?;

        if let Some(ref tokens) = self.syntax_tokens {
            for s in syntax_highlight(&self.input, tokens) {
                write!(f, "{}", s)?;
            }
        } else {
            // if no syntax tokens are available, just print the input
            write!(f, "{}", self.input)?;
        }

        write!(
            f,
            "  {}\r{}",
            self.hint.dimmed(),
            cursor::Right((PROMPT.len() + self.input.len() - self.cur_offset) as _)
        )
    }
}
