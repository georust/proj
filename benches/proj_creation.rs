use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use proj::Proj;

// Cost of constructing a transformation object. Each `Proj::new` creates its own PROJ context,
// so this is dominated by context setup rather than the projection itself.
// See https://github.com/georust/proj/issues/256
fn proj_creation(c: &mut Criterion) {
    c.bench_function("Proj::new EPSG:4326", |b| {
        b.iter(|| Proj::new(black_box("EPSG:4326")).unwrap());
    });
}

criterion_group!(benches, proj_creation);
criterion_main!(benches);
