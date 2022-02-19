use std::io::stdin;
use termion::event::Key;
use termion::input::TermRead;

const HEADER: &'static str = "
---- smartcalc ----
";

mod prompt;
mod spinner;
mod thread_loop;

use prompt::Prompt;

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

            Key::Char('\n') => {
                let mut ps = ps.lock();
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
