# Utility to sync directory

Add `watcher` to your `Cargo.toml`:
```toml
dir_watcher = { git = "https://github.com/TENX-S/dir_watcher", branch = "main" }
```
## Examples

```shell
$ cargo run --release --example --features "event" /absolute/path/of/directory/you/want/to/watch
```

Then try to move or add entries at the directory you specified, it will print your operations

### License

This project is licensed under the MIT license.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in dir_watcher by you, shall be licensed as MIT, without any additional terms or conditions.
