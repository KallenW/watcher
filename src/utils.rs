macro_rules! ls {
    ($location: expr, $pattern: expr) => {{
        let location = ::std::path::PathBuf::from($location);
        let mut targets = vec![];
        for p in $pattern {
            let mut temp = location.clone();
            temp.push(p);
            targets.push(temp);
        }

        let mut snapshots = HashSet::new();
        for t in targets {
            snapshots = snapshots.union(&::glob::glob(t.to_str().unwrap()).unwrap().filter_map(Result::ok).collect()).into_iter().cloned().collect();
        }
        snapshots

    }};
}
