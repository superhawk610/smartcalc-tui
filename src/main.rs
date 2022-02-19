use colored::*;
use std::fmt::Write as _;
use std::io::{stdin, stdout, Write};
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const HEADER: &'static str = "
---- smartcalc ----
";

fn main() {
    if let Err(reason) = do_main() {
        eprintln!("whoops! {:?}", reason);
        std::process::exit(1);
    }
}

fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let mut stdin = stdin().keys();
    let prompt = Prompt::spawn();

    // {
    //     let ps = Arc::downgrade(&prompt.state());
    //     std::thread::spawn(move || {
    //         let mut spinner = Spinner::new();
    //         while let Some(text) = spinner.recv() {
    //             match ps.upgrade() {
    //                 Some(ps) => ps.lock().set_hint(text),
    //                 None => {
    //                     spinner.stop();
    //                     break;
    //                 }
    //             }
    //         }
    //     })
    // };

    let ps = prompt.state();
    while let Some(Ok(k)) = stdin.next() {
        match k {
            Key::Ctrl('c') => {
                break;
            }

            Key::Char(c) if c.is_ascii() => ps.lock().insert_input(c),

            Key::Left => ps.lock().cursor_left(),
            Key::Right => ps.lock().cursor_right(),

            Key::Backspace => ps.lock().delete_backward(),
            Key::Delete => ps.lock().delete_forward(),

            // TODO: implement command history
            // Key::Up => { }
            // Key::Down => { }
            _ => (),
        }
    }

    // cleanup
    prompt.stop();

    Ok(())
}

fn print_header() {
    println!("{}", HEADER);
}

use parking_lot::Mutex;
use std::sync::Arc;

struct Prompt {
    thread_loop: ThreadLoop,
    state: Arc<Mutex<PromptState>>,
}

struct PromptState {
    pub cur_offset: usize,
    pub input: String,
    pub hint: String,
}

impl Prompt {
    const PROMPT: &'static str = "> ";

    fn spawn() -> Self {
        let state = Arc::new(Mutex::new(PromptState::new()));

        let thread_loop = {
            let stdout = stdout();
            let mut stdout = stdout.into_raw_mode().unwrap();
            let state = Arc::clone(&state);
            ThreadLoop::spawn(50, move || {
                let state = state.lock();

                write!(
                    stdout,
                    "\r{}{}{}  {}\r{}",
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

    fn state(&self) -> Arc<Mutex<PromptState>> {
        Arc::clone(&self.state)
    }

    fn stop(self) {
        print!("\r\n");
        self.thread_loop.stop();
    }
}

impl PromptState {
    fn new() -> Self {
        Self {
            cur_offset: 0,
            input: String::with_capacity(1024),
            hint: String::with_capacity(1024),
        }
    }

    fn append_input(&mut self, input: &str) {
        write!(self.input, "{}", input).unwrap();
    }

    fn insert_input(&mut self, c: char) {
        let idx = self.input.len() - self.cur_offset;
        self.input.insert(idx, c);
    }

    fn set_hint(&mut self, hint: &str) {
        self.hint.clear();
        write!(self.hint, "{}", hint).unwrap();
    }

    fn cursor_left(&mut self) {
        if self.cur_offset < self.input.len() {
            self.cur_offset += 1;
        }
    }

    fn cursor_right(&mut self) {
        if self.cur_offset > 0 {
            self.cur_offset -= 1;
        }
    }

    fn delete_backward(&mut self) {
        let cursor_idx = self.input.len() - self.cur_offset;
        if cursor_idx > 0 {
            self.input.remove(cursor_idx - 1);
        }
    }

    fn delete_forward(&mut self) {
        if self.cur_offset > 0 {
            self.input.remove(self.input.len() - self.cur_offset);
            self.cur_offset -= 1;
        }
    }
}

use crossbeam_channel::{unbounded, Receiver, Sender};

struct Spinner {
    thread_loop: ThreadLoop,
    rx: Receiver<&'static str>,
}

impl Spinner {
    const STATES: [&'static str; 4] = ["1", "2", "3", "4"];

    fn recv(&mut self) -> Option<&'static str> {
        self.rx.recv().ok()
    }

    fn new() -> Self {
        let (tx, rx) = unbounded();
        let mut state = 0;
        let thread_loop = ThreadLoop::spawn(250, move || {
            state = (state + 1) % Self::STATES.len();
            tx.send(Self::STATES[state]).unwrap();
        });

        Self { thread_loop, rx }
    }

    fn stop(self) {
        self.thread_loop.stop();
    }
}

struct ThreadLoop {
    handle: std::thread::JoinHandle<()>,
    done: Sender<()>,
}

impl ThreadLoop {
    fn spawn<F>(tick_rate: u64, mut f: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let (tx, rx) = unbounded();
        let handle = std::thread::spawn(move || loop {
            if let Ok(()) = rx.try_recv() {
                break;
            }

            f();
            std::thread::sleep(std::time::Duration::from_millis(tick_rate));
        });

        Self { handle, done: tx }
    }

    fn stop(self) {
        self.done.send(()).unwrap();
        self.handle.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
