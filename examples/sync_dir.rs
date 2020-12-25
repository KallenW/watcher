use watcher::Watcher;
use std::sync::Arc;
use std::thread;

fn main() {

    let target = std::env::args().skip(1).next().unwrap();
    let watcher = Arc::new(Watcher::new(&target).unwrap().depth(2));

    let update_watcher = Arc::clone(&watcher);

    thread::spawn(move || update_watcher.keep_sync_with_idle(None));

    let mut previous_length = watcher.get_snapshot().len();
    let dest = watcher.get_target();
    println!("Number of entries at {}: {}", dest, previous_length);
    loop {
        let current_length = watcher.get_snapshot().len();
        if current_length != previous_length {
            println!("\nNumber of entries at {}: {}", dest, current_length);
            previous_length = current_length;
            println!("Events from the beginning: {:#?}", watcher.get_events());
        }
    }
}
