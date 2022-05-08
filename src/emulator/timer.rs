use std::{thread, time};

pub struct Timer<T: FnMut() -> bool> {
    pub interval: time::Duration,
    pub action: T,
}

impl<T: FnMut() -> bool> Timer<T> {
    pub fn run(&mut self) -> () {
        loop {
            if !(self.action)() {
                return;
            }
            thread::sleep(self.interval)
        }
    }
}

use std::sync::Arc;
use atomic_counter::{RelaxedCounter, AtomicCounter};

pub struct ThreadedCounter {
    pub counter: Arc<atomic_counter::RelaxedCounter>,
    pub handle: std::thread::JoinHandle<()>,
}

pub fn create_threaded_counter(interval: time::Duration) -> ThreadedCounter {
    let val = Arc::new(RelaxedCounter::new(255));

    let val_clone = val.clone();
    let mut timer = Timer {
        interval,
        action: move || {
            if (*val_clone).get() < 255 {
                val_clone.inc();
                return true;
            }
            else {
                return (val_clone).get() < 256;
            }
        },
    };
    let handle = std::thread::spawn(move || timer.run());
    return ThreadedCounter {
        counter: val.clone(),
        handle,
    };
}
