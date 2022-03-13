use colored::*;
use const_format::formatcp;
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use std::sync::Arc;

// TODO: display smartcalc version as well as smartcalc-tui
// (probably want built::util::parse_versions https://docs.rs/built/latest/built/)
const VERSION: &str = env!("CARGO_PKG_VERSION");

const HEADER: &str = formatcp!(
    "
---- smartcalc {} ----
(ctrl+C / ctrl+D to quit)
",
    VERSION
);

mod calculate;
mod prompt;
mod syntax;
mod thread_loop;

use calculate::Calculate;
use prompt::Prompt;

pub fn spawn() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", HEADER);

    let prompt = Prompt::spawn();

    {
        let ps = Arc::downgrade(&prompt.state());
        std::thread::spawn(move || {
            let mut calc = Calculate::default();

            loop {
                match ps.upgrade() {
                    None => {
                        break;
                    }
                    Some(ps) => {
                        // TODO: memoize
                        let mut ps = ps.lock();
                        // drop(ps) then relock after execution?
                        if let Some((res, tokens)) = calc.execute(&ps.input) {
                            ps.set_hint(&res);
                            ps.set_syntax_tokens(tokens);
                        } else {
                            ps.clear_hint();
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });
    }

    let ps = prompt.state();

    while let Event::Key(key_event) = read()? {
        let modifier = key_event.modifiers;
        match key_event.code {
            KeyCode::Char('C') | KeyCode::Char('D') if modifier == KeyModifiers::CONTROL => {
                let mut ps = ps.lock();

                // erase hint then flush
                ps.clear_hint();
                print!("{}", ps);
                break;
            }
            KeyCode::Enter => {
                let mut ps = ps.lock();

                // erase hint then flush
                let hint = ps.hint.clone();
                ps.clear_hint();
                ps.cur_offset = 0;
                if !hint.is_empty() {
                    print!("{} {} {}", ps, "=>".dimmed(), hint.green());
                }

                // move to next line
                ps.clear_input();
                println!();
            }
            KeyCode::Char(c) if c.is_ascii() => ps.lock().insert_input(c),
            KeyCode::Left => ps.lock().cursor_left(),
            KeyCode::Right => ps.lock().cursor_right(),
            KeyCode::Backspace => ps.lock().delete_backward(),
            KeyCode::Delete => ps.lock().delete_forward(),
            _ => (),
        }
    }

    // cleanup
    prompt.stop();

    Ok(())
}
