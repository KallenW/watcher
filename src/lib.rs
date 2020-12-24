//! Utility to sync directory
use anyhow::{Result, bail};
use std::path::PathBuf;
use glob::glob;
use std::sync::{RwLock, Mutex};
use Event::*;

const DEFAULT_SYNC_IDLE: u64 = 200;

#[derive(Debug, Clone)]
pub enum Event<TIME=String> {
    Add(Vec<PathBuf>, TIME),
    Remove(Vec<PathBuf>, TIME),
}

#[derive(Debug)]
pub struct Watcher {
    depth: u64,
    pub target: String,
    snapshot: RwLock<Vec<PathBuf>>,
    events: Mutex<Vec<Event>>,
}

#[inline(always)]
fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}

macro_rules! ls {
    ($target: expr) => {
        glob(&$target).unwrap().filter_map(Result::ok).collect::<Vec<_>>()
    };
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

impl Watcher {

    #[inline(always)]
    pub fn new(target: &str) -> Result<Self> {
        if PathBuf::from(target).is_absolute() {
            Ok(Watcher {
                depth: 1,
                target: target.to_owned(),
                snapshot: RwLock::new(ls!(target)),
                events: Mutex::new(Vec::<Event>::new()),
            })
        } else {
            bail!("Watcher must be initialized with an absolute path!")
        }
    }

    #[inline(always)]
    pub fn depth(mut self, depth: u64) -> Self {
        self.depth = depth;
        let mut target = PathBuf::from(&self.target);
        for _ in 0..depth {
            target.push("*");
        }
        self.target = target.to_str().unwrap().to_owned();
        self.snapshot = RwLock::new(ls!(&self.target));
        self
    }

    pub fn sync_once(&self) {

        let previous = self.snapshot.read().unwrap().clone(); // This unwrap will never panic

        if let Ok(mut latest) = self.snapshot.try_write() {
            *latest = ls!(self.target.as_str());

            record_events!(removed, &previous, latest);
            record_events!(added, latest.iter(), previous);

            if !removed.is_empty() {
                if let Ok(mut push_event) = self.events.try_lock() {
                    push_event.push(Remove(removed, now()));
                }
            }

            if !added.is_empty() {
                if let Ok(mut push_event) = self.events.try_lock() {
                    push_event.push(Add(added, now()));
                }
            }

        }

    }

    #[inline(always)]
    pub fn keep_sync_with_idle(&self, idle_ms: Option<u64>) -> ! {
        loop {
            // WE MUST HAVE AN IDLE HERE!
            // Or it may lead to a performance problem because wasting too much CPU time
            // when the update operation occurs only occasionally
            std::thread::sleep(std::time::Duration::from_millis(idle_ms.unwrap_or(DEFAULT_SYNC_IDLE)));
            self.sync_once();
        }
    }

    #[inline(always)]
    pub fn get_snapshot(&self) -> Vec<PathBuf> {
        self.snapshot.read().unwrap().clone()
    }

    #[inline(always)]
    pub fn get_events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }

}
