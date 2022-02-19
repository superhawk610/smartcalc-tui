use crossbeam_channel::{unbounded, Sender};

pub struct ThreadLoop {
    handle: std::thread::JoinHandle<()>,
    done: Sender<()>,
}

impl ThreadLoop {
    pub fn spawn<F>(tick_rate: u64, mut f: F) -> Self
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

    pub fn stop(self) {
        self.done.send(()).unwrap();
        self.handle.join().unwrap();
    }
}
