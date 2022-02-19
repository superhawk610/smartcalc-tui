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
    let mut prompt = Prompt::new();
    prompt.start();

    // let mut spinner = Spinner::new();
    // spinner.spin();

    // let spin_ch = spinner.ch().as_ref().unwrap();
    // while let Ok(text) = spin_ch.recv() {
    //     prompt.set_hint(text);
    // }

    while let Some(Ok(k)) = stdin.next() {
        match k {
            Key::Ctrl('c') | Key::Char('q') => {
                break;
            }
            Key::Char(c) => {
                // TODO: allow symbols?
                if ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == ' ' {
                    // TODO: insert at len() - cur_offset
                    prompt.append_input(&format!("{}", c));
                }
            }
            Key::Left => {
                prompt.set_hint("pressed left!");
                prompt.cursor_left();
            }
            Key::Right => {
                prompt.set_hint("pressed right!");
                prompt.cursor_right();
            }
            _ => (),
        }
    }

    // cleanup
    prompt.stop();
    // spinner.stop();

    Ok(())
}

fn print_header() {
    println!("{}", HEADER);
}

use parking_lot::Mutex;
use std::sync::Arc;

// FIXME: move locking out of prompt and handle at runtime
struct Prompt {
    thread: Option<std::thread::JoinHandle<()>>,
    contents: Arc<Mutex<PromptContents>>,
    done: Option<Sender<()>>,
}

struct PromptContents {
    cur_offset: usize,
    input: String,
    hint: String,
}

impl Prompt {
    const PROMPT: &'static str = "> ";

    fn new() -> Self {
        Self {
            contents: Arc::new(Mutex::new(PromptContents::new())),
            done: None,
            thread: None,
        }
    }

    fn append_input(&self, input: &str) {
        let mut contents = self.contents.lock();
        write!(contents.input, "{}", input).unwrap();
    }

    fn set_hint(&self, hint: &str) {
        let mut contents = self.contents.lock();
        contents.hint.clear();
        write!(contents.hint, "{}", hint).unwrap();
    }

    fn cursor_left(&self) {
        let mut contents = self.contents.lock();
        if contents.cur_offset < contents.input.len() {
            contents.cur_offset += 1;
        }
    }

    fn cursor_right(&self) {
        let mut contents = self.contents.lock();
        if contents.cur_offset > 0 {
            contents.cur_offset -= 1;
        }
    }

    fn start(&mut self) {
        let (tx, rx) = unbounded();
        self.done = Some(tx);

        let contents = self.contents.clone();
        let handle = std::thread::spawn(move || {
            let stdout = stdout();
            let mut stdout = stdout.lock().into_raw_mode().unwrap();

            loop {
                if let Ok(()) = rx.try_recv() {
                    write!(stdout, "\r\n").unwrap();
                    stdout.flush().unwrap();
                    break;
                }

                let c = contents.lock();

                write!(
                    stdout,
                    "\r{}{}{}  {}\r{}",
                    termion::clear::CurrentLine,
                    Self::PROMPT,
                    c.input,
                    c.hint.dimmed(),
                    cursor::Right((Self::PROMPT.len() + c.input.len() - c.cur_offset) as _)
                )
                .unwrap();

                stdout.flush().unwrap();

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        self.thread = Some(handle);
    }

    fn stop(&mut self) {
        if let Some(done) = self.done.take() {
            done.send(()).unwrap();

            // join the thread to make sure the RawTerminal has time to drop
            self.thread.take().unwrap().join().unwrap();
        }
    }
}

impl PromptContents {
    fn new() -> Self {
        Self {
            cur_offset: 0,
            input: String::with_capacity(1024),
            hint: String::with_capacity(1024),
        }
    }
}

use crossbeam_channel::{select, tick, unbounded, Receiver, Sender};

struct Spinner {
    done: Option<Sender<()>>,
    rx: Option<Receiver<&'static str>>,
}

impl Spinner {
    const STATES: [&'static str; 4] = ["1", "2", "3", "4"];

    fn new() -> Self {
        Self {
            done: None,
            rx: None,
        }
    }

    fn ch(&self) -> &Option<Receiver<&'static str>> {
        &self.rx
    }

    fn spin(&mut self) {
        let (tx, rx) = unbounded();
        self.rx = Some(rx);

        let (done_tx, done_rx) = unbounded();
        self.done = Some(done_tx);

        std::thread::spawn(move || {
            let ticker = tick(std::time::Duration::from_millis(250));
            let mut state = 0;

            loop {
                select! {
                    recv(done_rx) -> _ => {
                        break;
                    },
                    recv(ticker) -> _ => {
                        state = (state + 1) % Self::STATES.len();
                        tx.send(Self::STATES[state]).unwrap();
                    }
                }
            }
        });
    }

    fn stop(&mut self) {
        if let Some(done) = self.done.take() {
            done.send(()).unwrap();
        }
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
