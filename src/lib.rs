//! Utility to sync directory
use anyhow::{Result, bail};
use std::path::PathBuf;
use glob::glob;
use std::sync::{RwLock, Mutex};
use Event::*;

#[derive(Debug, Clone)]
pub enum Event<T=String> {
    Add(Vec<PathBuf>, T),
    Remove(Vec<PathBuf>, T),
}

#[derive(Debug)]
pub struct Watcher {
    target: String,
    snapshot: RwLock<Vec<PathBuf>>,
    events: Mutex<Vec<Event>>,
}

#[inline(always)]
fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}

macro_rules! ls {
    ($target: expr) => {
        glob([$target, "*"].iter().collect::<PathBuf>().to_str().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
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
                target: target.to_owned(),
                snapshot: RwLock::new(ls!(target)),
                events: Mutex::new(Vec::<Event>::new())
            })
        } else {
            bail!("Watcher must be initialized with an absolute path!")
        }
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

    pub fn keep_sync_with_idle(&self, idle_ms: u64) -> ! {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(idle_ms));
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
