macro_rules! sort_dedup {
    ($slice: expr) => {
        $slice.sort();
        $slice.dedup()
    };
}

macro_rules! ls {
    ($location: expr, $pattern: expr) => {{
        let mut entries = vec![];
        for p in $pattern {
            let mut temp = $location.clone();
            temp.push(p);
            entries.push(temp);
        }

        let mut snapshot = vec![];

        for e in entries {
            snapshot.extend_from_slice(&::glob::glob(e.to_str().unwrap()).unwrap().filter_map(Result::ok).collect::<Vec<_>>());
        }
        sort_dedup!(snapshot);
        snapshot

    }};
}
