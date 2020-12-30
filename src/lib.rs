//! Utility to sync directory
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

#[macro_use]
mod utils;
mod error;

use parking_lot::Mutex;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
use error::{anyhow, Result, InitDirWatcherError::*};
#[cfg(feature = "event")]
use Event::*;

const DEFAULT_SYNC_IDLE: u64 = 1;
type WatchingThread = Mutex<Option<JoinHandle<()>>>;

#[derive(Debug)]
pub struct DirWatcher {
    #[doc(hidden)]
    inner: Arc<_Watcher>,
    on_loop: Arc<AtomicBool>,
    wthread: WatchingThread,
}

impl DirWatcher {

    /// You can use [pattern in glob](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html) here
    #[inline(always)]
    pub fn new(target: &str, pattern: &[&str]) -> Self {
        DirWatcher {
            inner: Arc::new(_Watcher::new(target, pattern).unwrap()),
            on_loop: Arc::new(AtomicBool::new(false)),
            wthread: Mutex::new(None),
        }
    }

    /// Sync once for the current target
    #[inline(always)]
    pub fn refresh(&self) {
        self.inner.sync_once();
    }

    /// Spawn a new thread to watch the target directory
    pub fn watch_on_idle(&self, idle_ns: Option<u64>) {
        ON_LOOP.store(true, SeqCst);
        let update = Arc::clone(&self.inner);
        let on_loop = Arc::clone(&self.on_loop);
        *self.wthread.lock() = Some(thread::spawn(move || {
            loop {
                // WE MUST HAVE AN IDLE HERE!
                // Or it may lead to a performance problem because of wasting too much CPU time
                // when the update operation occurs only occasionally
                if !on_loop.load(SeqCst) {
                    thread::park();
                }
                thread::sleep(std::time::Duration::from_nanos(idle_ns.unwrap_or(DEFAULT_SYNC_IDLE)));
                update.sync_once();
            }
        }));
    }

    /// Let a watcher pause the watching, pause a watcher which already paused will have no effect
    #[inline(always)]
    pub fn pause(&self) {
        self.on_loop.store(false, SeqCst);
    }

    /// Let a watcher resume the watching, use it on a watcher which is running will have no effect
    #[inline(always)]
    pub fn resume(&self) {
        self.on_loop.store(true, SeqCst);
        if let Some(wthread) = self.wthread.lock().as_ref() {
            wthread.thread().unpark();
        }
    }

    /// Return a current snapshot of the target directory
    #[inline(always)]
    pub fn current(&self) -> Vec<PathBuf> {
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
    pub fn get_target(&self) -> (&str, &[String]) {
        self.inner.get_target()
    }

    /// Return the watching state
    #[inline(always)]
    pub fn is_watching(&self) -> bool {
        self.on_loop.load(SeqCst)
    }

}

/// Represents the operations that cause changes in the directory
#[derive(Debug, Clone)]
#[cfg(feature = "event")]
pub enum Event {
    Add(Vec<PathBuf>),
    Remove(Vec<PathBuf>),
}

#[cfg(feature = "event")]
macro_rules! record_events {
    ($records: ident, $previous: expr, $updated: expr, $events: expr, $operations: tt) => {
        let mut $records = vec![];
        for x in $previous { if !$updated.contains(x) { $records.push(x.clone()); } }
        if !$records.is_empty() { $events.push($operations($records)); }
    };
}

#[doc(hidden)]
#[derive(Debug)]
struct _Watcher {
    target: _Target,
    snapshot: Mutex<Vec<PathBuf>>,
    #[cfg(feature = "event")]
    events: Mutex<Vec<Event>>,
}

impl _Watcher {

    #[inline(always)]
    fn new(location: &str, pattern: &[&str]) -> Result<Self> {
        Ok(_Watcher {
            target: _Target::new(location, pattern)?,
            snapshot: Mutex::new(ls!(PathBuf::from(location), pattern)),
            #[cfg(feature = "event")]
            events: Mutex::new(Vec::<Event>::new()),
        })
    }

    fn sync_once(&self) {
        let mut snapshot = self.snapshot.lock();
        #[cfg(feature = "event")]
        let previous = snapshot.clone();
        // Update the snapshot
        *snapshot = ls!(self.target.location.clone(), &self.target.pattern);
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
    fn get_target(&self) -> (&str, &[String]) {
        (self.target.location.to_str().unwrap(), &self.target.pattern)
    }

}

#[doc(hidden)]
#[derive(Debug)]
struct _Target {
    location: PathBuf,
    pattern: Vec<String>,
}

impl _Target {

    #[inline(always)]
    fn new(location: &str, pattern: &[&str]) -> Result<Self> {
        let location = PathBuf::from(location);
        let mut pattern = pattern.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        sort_dedup!(pattern);
        if !location.is_absolute() {
            Err(anyhow!(NonAbsPath))
        } else if !location.exists() {
            Err(anyhow!(InExistence))
        } else if !location.is_dir() {
            Err(anyhow!(NotADirectory))
        } else {
            Ok(_Target {
                location,
                pattern,
            })
        }
    }

}
