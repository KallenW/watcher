use watcher::Watcher;
use std::sync::Arc;
use std::thread;


fn main() {

    let watcher = Arc::new(Watcher::new("/Users/tenx/Music/MyMusic").unwrap());

    let update_watcher = Arc::clone(&watcher);

    let refreshing = thread::spawn(move || loop {
        update_watcher.sync_once();
    });

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
