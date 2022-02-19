use colored::*;
use const_format::formatcp;
use std::io::stdin;
use std::sync::Arc;
use termion::event::Key;
use termion::input::TermRead;

const LANG: &'static str = "en";

// TODO: display libsmartcalc version as well as smartcalc-tui
// (probably want built::util::parse_versions https://docs.rs/built/latest/built/)
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const HEADER: &'static str = formatcp!(
    "
---- smartcalc {} ----
(ctrl+C / ctrl+D to quit)
",
    VERSION
);

mod prompt;
mod spinner;
mod thread_loop;

use prompt::Prompt;
use spinner::Spinner;

pub fn spawn() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", HEADER);

    let mut stdin = stdin().keys();
    let prompt = Prompt::spawn();

    // {
    //     let ps = Arc::downgrade(&prompt.state());
    //     std::thread::spawn(move || {
    //         let mut spinner = Spinner::new();
    //         while let Some(text) = spinner.recv() {
    //             match ps.upgrade() {
    //                 Some(ps) => ps.lock().set_hint(&format!("{} loading...", text)),
    //                 None => {
    //                     spinner.stop();
    //                     break;
    //                 }
    //             }
    //         }
    //     });
    // }

    {
        let ps = Arc::downgrade(&prompt.state());
        std::thread::spawn(move || {
            use libsmartcalc::app::{ExecuteLine, ExecuteLineResult, SmartCalc};
            use libsmartcalc::executer::initialize;
            use num_format::SystemLocale;

            initialize();

            let mut app = SmartCalc::default();
            let locale = SystemLocale::default().unwrap();
            app.config.decimal_seperator = locale.decimal().to_string();
            app.config.thousand_separator = locale.separator().to_string();

            loop {
                match ps.upgrade() {
                    None => {
                        break;
                    }
                    Some(ps) => {
                        // TODO: memoize
                        // TODO: incorporate spinner
                        // TODO: syntax highlighting
                        // TODO: variable definition
                        let mut ps = ps.lock();
                        // drop(ps) then relock after execution?
                        let res = app.execute(LANG, ps.input.clone());

                        match res.lines[0] {
                            Some(ExecuteLine {
                                result: Ok(ExecuteLineResult { ref output, .. }),
                                ..
                            }) => ps.set_hint(output),
                            _ => ps.clear_hint(),
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });
    }

    let ps = prompt.state();
    while let Some(Ok(k)) = stdin.next() {
        match k {
            Key::Ctrl('c') | Key::Ctrl('d') => {
                let mut ps = ps.lock();

                // erase hint then flush
                ps.clear_hint();
                print!("{}", ps);

                break;
            }

            Key::Char('\n') => {
                let mut ps = ps.lock();

                // erase hint then flush
                let hint = ps.hint.clone();
                ps.clear_hint();
                if hint.len() > 0 {
                    print!("{} {} {}", ps, "=>".dimmed(), hint.green());
                }

                // move to next line
                ps.clear_input();
                print!("\n");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
