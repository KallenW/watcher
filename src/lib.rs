//! Utility to sync directory
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

#[macro_use]
mod utils;
mod error;

use parking_lot::Mutex;
#[cfg(feature = "event")]
use Event::*;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
use std::thread::{self, JoinHandle};
use error::{anyhow, Result, InitDirWatcherError::*};

const DEFAULT_SYNC_IDLE: u64 = 1;
const ON_LOOP: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct DirWatcher {
    #[doc(hidden)]
    inner: Arc<__Watcher>,
    on_loop: bool,
    wthread: Option<JoinHandle<()>>,
}

impl DirWatcher {

    /// You can use [pattern in glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html) here
    #[inline(always)]
    pub fn new(target: &str, pattern: &str) -> Self {
        DirWatcher {
            inner: Arc::new(__Watcher::new(target, pattern).unwrap()),
            on_loop: ON_LOOP.load(SeqCst),
            wthread: None,
        }
    }

    /// Sync once for the current target
    #[inline(always)]
    pub fn refresh(&self) {
        self.inner.sync_once();
    }

    /// Spawn a new thread to watch the target directory
    pub fn watch_with_idle(&mut self, idle_ns: Option<u64>) {
        ON_LOOP.store(true, SeqCst);
        let update = Arc::clone(&self.inner);
        self.wthread = Some(thread::spawn(move || {
            loop {
                // WE MUST HAVE AN IDLE HERE!
                // Or it may lead to a performance problem because of wasting too much CPU time
                // when the update operation occurs only occasionally
                if ON_LOOP.load(SeqCst) {
                    thread::park();
                }
                std::thread::sleep(std::time::Duration::from_nanos(idle_ns.unwrap_or(DEFAULT_SYNC_IDLE)));
                update.sync_once();
            }
        }));
    }

    #[inline(always)]
    pub fn pause(&self) {
        ON_LOOP.store(false, SeqCst);
    }

    #[inline(always)]
    pub fn resume(&self) {
        ON_LOOP.store(true, SeqCst);
        self.wthread.as_ref().unwrap().thread().unpark();
    }

    /// Return a current snapshot of the target directory
    #[inline(always)]
    pub fn get_snapshot(&self) -> Vec<PathBuf> {
        self.inner.get_snapshot()
    }

    /// Return events From the very beginning of watcher
    #[inline(always)]
    #[cfg(feature = "event")]
    pub fn get_events(&self) -> Vec<Event> {
        self.inner.get_events()
    }

    /// Return the target of watcher
    #[inline(always)]
    pub fn get_target(&self) -> &str {
        self.inner.get_target()
    }

    #[inline(always)]
    pub fn is_watching(&self) -> bool {
        ON_LOOP.load(SeqCst)
    }

}

/// Represents the operations that cause changes in the directory
#[derive(Debug, Clone)]
#[cfg(feature = "event")]
pub enum Event {
    Add(Vec<PathBuf>),
    Remove(Vec<PathBuf>),
}

#[doc(hidden)]
#[derive(Debug)]
struct __Watcher {
    target: PathBuf,
    snapshot: Mutex<Vec<PathBuf>>,
    #[cfg(feature = "event")]
    events: Mutex<Vec<Event>>,
}

#[cfg(feature = "event")]
macro_rules! record_events {
    ($records: ident, $previous: expr, $updated: expr, $events: expr, $operations: tt) => {
        let mut $records = vec![];
        for x in $previous { if !$updated.contains(x) { $records.push(x.clone()); } }
        if !$records.is_empty() { $events.push($operations($records)); }
    };
}

impl __Watcher {

    #[inline(always)]
    fn new(target: &str, pattern: &str) -> Result<Self> {
        let mut target = PathBuf::from(target);

        if !target.is_absolute() {
            Err(anyhow!(NonAbsPath))
        } else if !target.exists() {
            Err(anyhow!(InExistence))
        } else if !target.is_dir() {
            Err(anyhow!(NotADirectory))
        } else {
            target.push(pattern);
            let snapshot = Mutex::new(ls!(target.to_str().unwrap()));
            Ok(__Watcher {
                target,
                snapshot,
                #[cfg(feature = "event")]
                events: Mutex::new(Vec::<Event>::new()),
            })
        }
    }

    fn sync_once(&self) {
        let mut snapshot = self.snapshot.lock();
        #[cfg(feature = "event")]
        let previous = snapshot.clone();
        *snapshot = ls!(self.target.to_str().unwrap());
        #[cfg(feature = "event")]
        {
            let mut events = self.events.lock();
            record_events!(removed, &previous, snapshot, events, Remove);
            record_events!(added, snapshot.iter(), previous, events, Add);
        }
    }

    #[inline(always)]
    fn get_snapshot(&self) -> Vec<PathBuf> {
        self.snapshot.lock().clone()
    }

    #[inline(always)]
    #[cfg(feature = "event")]
    fn get_events(&self) -> Vec<Event> {
        self.events.lock().clone()
    }

    #[inline(always)]
    fn get_target(&self) -> &str {
        self.target.to_str().unwrap()
    }

}
