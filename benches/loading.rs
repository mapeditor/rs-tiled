use criterion::{Criterion, criterion_group, criterion_main};
use tiled::Loader;

fn load(path: &str) {
    Loader::new().load_tmx_map(path).unwrap();
}

fn bench_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_tmx_map");
    group.bench_function("base64", |b| b.iter(|| load("assets/tiled_base64.tmx")));
    group.bench_function("base64_gzip", |b| {
        b.iter(|| load("assets/tiled_base64_gzip.tmx"))
    });
    group.bench_function("csv", |b| b.iter(|| load("assets/tiled_csv.tmx")));
    group.bench_function("base64_zlib_infinite", |b| {
        b.iter(|| load("assets/tiled_base64_zlib_infinite.tmx"))
    });
    group.finish();
}

criterion_group!(benches, bench_loading);
criterion_main!(benches);
