use criterion::{Criterion, criterion_group, criterion_main};
use tiled::{Layer, LayerType, Loader};

/// Recursively counts every layer reachable from `layers`, descending into group layers.
fn count_layers<'map>(layers: impl Iterator<Item = Layer<'map>>) -> usize {
    let mut count = 0;
    for layer in layers {
        count += 1;
        if let LayerType::Group(group) = layer.layer_type() {
            count += count_layers(group.layers());
        }
    }
    count
}

fn bench_get_tile(c: &mut Criterion) {
    let map = Loader::new()
        .load_tmx_map("assets/tiled_base64.tmx")
        .unwrap();
    let layer = map
        .layers()
        .find_map(|layer| layer.as_tile_layer())
        .expect("fixture has a tile layer");
    let width = layer.width().unwrap();
    let height = layer.height().unwrap();

    c.bench_function("get_tile/full_layer_scan", |b| {
        b.iter(|| {
            for y in 0..height as i32 {
                for x in 0..width as i32 {
                    std::hint::black_box(layer.get_tile(x, y));
                }
            }
        })
    });
}

fn bench_enumerate_layers(c: &mut Criterion) {
    let map = Loader::new()
        .load_tmx_map("assets/tiled_group_layers.tmx")
        .unwrap();

    c.bench_function("enumerate_layers/group_layers", |b| {
        b.iter(|| std::hint::black_box(count_layers(map.layers())))
    });
}

criterion_group!(benches, bench_get_tile, bench_enumerate_layers);
criterion_main!(benches);
