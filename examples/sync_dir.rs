use dir_watcher::DirWatcher;
use anyhow::Result;

fn main() -> Result<()> {

    let target = std::env::args().skip(1).next().unwrap();
    let watcher = DirWatcher::new(&target, &["*"])?;
    watcher.loop_watch()?;

    let mut previous_length = watcher.get_snapshot().len();
    let dest = watcher.get_target();
    println!("Number of entries of {} - {:?}: {}", dest.0.display(), dest.1, previous_length);
    loop {
        let current_length = watcher.get_snapshot().len();
        if current_length != previous_length {
            println!("\nNumber of entries of {} - {:?}: {}", dest.0.display(), dest.1, current_length);
            previous_length = current_length;
            println!("Events from the beginning: {:#?}", watcher.get_events());
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
