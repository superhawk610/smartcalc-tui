use smartcalc_tui::spawn as spawn_tui;

fn main() {
    if let Err(reason) = spawn_tui() {
        eprintln!("whoops! {:?}", reason);
        std::process::exit(1);
    }
}
