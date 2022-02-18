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

const PROMPT: &'static str = "> ";
const REFRESH_RATE_MS: u64 = 50;

fn main() {
    if let Err(reason) = do_main() {
        eprintln!("whoops! {:?}", reason);
        std::process::exit(1);
    }
}

fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode()?;
    let mut stdin = stdin().keys();

    // TODO: probably do this better
    let mut input = String::with_capacity(1024);
    let mut hint = String::with_capacity(1024);
    let mut cur_offset: usize = 0;

    print_promptline(&mut stdout, cur_offset, &input, &hint)?;
    stdout.flush()?;

    while let Some(Ok(k)) = stdin.next() {
        match k {
            Key::Ctrl('c') | Key::Char('q') => {
                write!(stdout, "\r\n")?;
                break;
            }
            Key::Char(c) => {
                // TODO: allow symbols?
                if ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == ' ' {
                    // TODO: insert at len() - cur_offset
                    write!(input, "{}", c)?;
                }
            }
            Key::Left => {
                hint.clear();
                write!(hint, "pressed left!")?;

                cur_offset += 1;
                if cur_offset >= input.len() {
                    cur_offset = input.len();
                }
            }
            Key::Right => {
                hint.clear();
                write!(hint, "pressed right!")?;

                cur_offset = match cur_offset.checked_sub(1) {
                    Some(n) => n,
                    None => 0,
                };
            }
            _ => (),
        }

        print_promptline(&mut stdout, cur_offset, &input, &hint)?;
        stdout.flush()?;
    }

    Ok(())
}

fn print_header() {
    println!("{}", HEADER);
}

fn print_promptline(
    mut stdout: impl Write,
    cur_offset: usize,
    input: &str,
    hint: &str,
) -> Result<(), std::io::Error> {
    write!(
        stdout,
        "\r{}{}{}  {}\r{}",
        termion::clear::CurrentLine,
        PROMPT,
        input,
        hint.dimmed(),
        cursor::Right((PROMPT.len() + input.len() - cur_offset) as _)
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
