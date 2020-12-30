use dir_watcher::DirWatcher;

fn main() {

    let target = std::env::args().skip(1).next().unwrap();
    let watcher = DirWatcher::new(&target, &["*.flac", "*.mp3", "*.txt"]);
    watcher.watch_on_idle(None);

    let mut previous_length = watcher.current().len();
    let dest = watcher.get_target();
    println!("Number of entries of {} - {:?}: {}", dest.0, dest.1, previous_length);
    loop {
        let current_length = watcher.current().len();
        if current_length != previous_length {
            println!("\nNumber of entries of {} - {:?}: {}", dest.0, dest.1, current_length);
            previous_length = current_length;
            println!("Events from the beginning: {:#?}", watcher.get_events());
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
