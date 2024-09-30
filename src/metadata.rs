use std::{fs::File, path::Path};

use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{MetadataOptions, Tag},
    probe::Hint,
};

// file opening here disturbs me
// in fact, this whole block disturbs me
// getting tags this way because I can leverage an existing dependency
// but near certain that there is a more efficient method
// tags are just metadata - it is unnecessary to read a file in full to retrieve
// expect a quick look would turn up crates for this purpose, alas

pub fn get_tags(path: &Path) -> Option<Vec<Tag>> {
    let source = Box::new(File::open(path).expect("box file error"));
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    hint.with_extension(path.extension()?.to_str().expect("hint error"));

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let mut probe = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    if let Some(meta) = probe.format.metadata().current() {
        let tags = meta.tags();
        if !tags.is_empty() {
            return Some(tags.to_owned());
        }
    } else if let Some(meta) = probe.metadata.get().as_ref().and_then(|m| m.current()) {
        let tags = meta.tags();
        if !tags.is_empty() {
            return Some(tags.to_owned());
        }
    }

    None
}
