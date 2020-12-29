use dir_watcher::DirWatcher;

fn main() {

    let target = std::env::args().skip(1).next().unwrap();
    let mut watcher = DirWatcher::new(&target, "*");
    watcher.watch_with_idle(None);

    let mut previous_length = watcher.get_snapshot().len();
    let dest = watcher.get_target();
    // println!("Number of entries at {}: {}", dest, previous_length);
    // println!("On watching: {}", watcher.is_watching());
    // println!("Pause for 10 secs!");
    // watcher.pause();
    // println!("On watching: {}", watcher.is_watching());
    // std::thread::sleep(std::time::Duration::from_secs(10));
    // println!("Time up! Resume!");
    // watcher.resume();
    // println!("On watching: {}", watcher.is_watching());
    loop {
        let current_length = watcher.get_snapshot().len();
        if current_length != previous_length {
            println!("\nNumber of entries at {}: {}", dest, current_length);
            previous_length = current_length;
            println!("Events from the beginning: {:#?}", watcher.get_events());
        }
    }
}
