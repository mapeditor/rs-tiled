#![no_main]

use libfuzzer_sys::fuzz_target;

use std::path::Path;

use tiled::{DefaultResourceCache, Loader, ResourceReader};

struct FuzzResourceReader<'a> {
    data: &'a [u8],
}

impl<'a> FuzzResourceReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        FuzzResourceReader { data }
    }
}

impl<'a> ResourceReader for FuzzResourceReader<'a> {
    type Resource = &'a [u8];
    type Error = std::io::Error;

    fn read_from(&mut self, _path: &Path) -> Result<Self::Resource, Self::Error> {
        Ok(self.data)
    }
}

fuzz_target!(|data: &[u8]| {
    let mut loader =
        Loader::with_cache_and_reader(DefaultResourceCache::new(), FuzzResourceReader::new(data));
    let _ = loader.load_tmx_map("fuzz.tmx");
});
