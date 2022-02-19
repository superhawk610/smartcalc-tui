use crate::thread_loop::ThreadLoop;
use crossbeam_channel::{unbounded, Receiver};

pub struct Spinner {
    thread_loop: ThreadLoop,
    rx: Receiver<&'static str>,
}

impl Spinner {
    const STATES: [&'static str; 8] = ["⣾", "⣷", "⣯", "⣟", "⡿", "⢿", "⣻", "⣽"];

    pub fn recv(&mut self) -> Option<&'static str> {
        self.rx.recv().ok()
    }

    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        let mut state = 0;
        let thread_loop = ThreadLoop::spawn(150, move || {
            state = (state + 1) % Self::STATES.len();
            tx.send(Self::STATES[state]).unwrap();
        });

        Self { thread_loop, rx }
    }

    pub fn stop(self) {
        self.thread_loop.stop();
    }
}
