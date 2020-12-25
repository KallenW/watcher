macro_rules! ls {
    ($target: expr) => {
        ::glob::glob(&$target).unwrap().filter_map(Result::ok).collect::<Vec<_>>()
    };
}
