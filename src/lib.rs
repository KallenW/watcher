//! Utility to sync directory
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

#[macro_use]
mod utils;
mod error;

use error::{Result, bail};
use parking_lot::Mutex;
use Event::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

const DEFAULT_SYNC_IDLE: u64 = 200;

#[derive(Debug)]
pub struct DirWatcher {
    #[doc(hidden)]
    inner: Arc<__Watcher>
}

impl DirWatcher {

    /// You can use [pattern in glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html) here
    pub fn new(target: &str) -> Self {
        DirWatcher {
            inner: Arc::new(__Watcher::new(target).unwrap())
        }
    }

    /// Spawn a new thread to watch the target directory
    pub fn keep_sync_with_idle(&self, idle_ms: Option<u64>) -> JoinHandle<()> {
        let update = Arc::clone(&self.inner);
        thread::spawn(move || {
            update.keep_sync_with_idle(idle_ms);
        })
    }

    /// Return a current snapshot of the target directory
    #[inline(always)]
    pub fn get_snapshot(&self) -> Vec<PathBuf> {
        self.inner.get_snapshot()
    }

    /// Return events From the very beginning of watcher
    #[inline(always)]
    pub fn get_events(&self) -> Vec<Event> {
        self.inner.get_events()
    }

    /// Return the target of watcher
    #[inline(always)]
    pub fn get_target(&self) -> &str {
        self.inner.get_target()
    }

}

/// Represents the operations that cause changes in the directory
#[derive(Debug, Clone)]
pub enum Event {
    Add(Vec<PathBuf>),
    Remove(Vec<PathBuf>),
}

#[doc(hidden)]
#[derive(Debug)]
struct __Watcher {
    target: PathBuf,
    snapshot: Mutex<Vec<PathBuf>>,
    events: Mutex<Vec<Event>>,
}

macro_rules! record_events {
    ($records: ident, $previous: expr, $updated: expr) => {
        let mut $records = vec![];

        for x in $previous {
            if !$updated.contains(x) {
                $records.push(x.clone());
            }
        }

    };
}

impl __Watcher {

    #[inline(always)]
    fn new(target: &str) -> Result<Self> {
        let mut target = PathBuf::from(target);

        if !target.is_absolute() {
            bail!("Watcher must be initialized with an absolute path!")
        } else if !target.exists() {
            bail!("Watcher must be initialized with an existed path!")
        } else if !target.is_dir() {
            bail!("Watcher must be initialized with a directory!")
        } else {
            target.push("*");
            Ok(__Watcher {
                target: target.clone(),
                snapshot: Mutex::new(ls!(target.to_str().unwrap())),
                events: Mutex::new(Vec::<Event>::new()),
            })
        }
    }

    fn sync_once(&self) {

        let previous = self.snapshot.lock().clone();

        if let Some(mut latest) = self.snapshot.try_lock() {
            *latest = ls!(self.target.to_str().unwrap());

            record_events!(removed, &previous, latest);
            record_events!(added, latest.iter(), previous);

            if !removed.is_empty() {
                if let Some(mut push_event) = self.events.try_lock() {
                    push_event.push(Remove(removed));
                }
            }

            if !added.is_empty() {
                if let Some(mut push_event) = self.events.try_lock() {
                    push_event.push(Add(added));
                }
            }

        }

    }

    #[inline(always)]
    fn keep_sync_with_idle(&self, idle_ms: Option<u64>) -> ! {
        loop {
            // WE MUST HAVE AN IDLE HERE!
            // Or it may lead to a performance problem because wasting too much CPU time
            // when the update operation occurs only occasionally
            std::thread::sleep(std::time::Duration::from_millis(idle_ms.unwrap_or(DEFAULT_SYNC_IDLE)));
            self.sync_once();
        }
    }

    #[inline(always)]
    fn get_snapshot(&self) -> Vec<PathBuf> {
        self.snapshot.lock().clone()
    }

    #[inline(always)]
    fn get_events(&self) -> Vec<Event> {
        self.events.lock().clone()
    }

    #[inline(always)]
    fn get_target(&self) -> &str {
        self.target.to_str().unwrap()
    }

}
