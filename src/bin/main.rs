use watcher::Watcher;
use std::sync::Arc;
use std::thread;

fn main() {

    let watcher = Arc::new(Watcher::new(&std::env::args().skip(1).next().unwrap()).unwrap());

    let update_watcher = Arc::clone(&watcher);

    thread::spawn(move || update_watcher.keep_sync_with_idle(200));

    let mut previous_length = watcher.get_snapshot().len();
    println!("{}", previous_length);
    loop {
        let current_length = watcher.get_snapshot().len();
        if current_length != previous_length {
            println!("{}", current_length);
            previous_length = current_length;
            println!("{:#?}", watcher.get_events());
        }
    }
}
