# `std::fs::File::sync_all` for a directory


Add `watcher` to your `Cargo.toml`:
```toml
watcher = { git = "https://github.com/TENX-S/watcher", branch = "main" }
```
## Examples

```shell
$ cargo run --release --example /absolute/path/of/directory/you/want/to/watch
```

Then try to move or add entries at the directory you specified, it will print your operations
