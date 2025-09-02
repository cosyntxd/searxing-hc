use backend::database::Database;
use criterion::{Criterion, criterion_group, criterion_main};
use std::{hint::black_box, path::Path};

const POPULAR_15: &'static str = "website game ai portfolio app ai bot for project tracker calculator discord learning python system";
const NICHE_15: &'static str = "fpga frc cad solder library ftc xrp SPY stock synthesizer sand simulator physics minecraft executable";
const ENG_15: &'static str = "a and the to for is with of in you that it this your on";
const FILE_PATH: &'static str = "/Users/ryan/Github/searxing-hc/complete_database.json";
fn bench_search(c: &mut Criterion) {
    // todo: relative
    let db = &mut Database::load_file(FILE_PATH);
    test_input(c, db, "0_blank_query", " ");

    test_input(c, db, "5_popular_projects", "website ai project html made");
    test_input(c, db, "5_niche_projects", "fpga frc cad solder library");
    test_input(c, db, "5_popular_english", "a and the to for");

    test_input(c, db, "15_popular_projects", POPULAR_15);
    test_input(c, db, "15_niche_projects", NICHE_15);
    test_input(c, db, "15_popular_english", ENG_15);

    let total_50 = format!("{} {} {} for a total of 50", POPULAR_15, NICHE_15, ENG_15);
    test_input(c, db, "50_total", total_50);
}

fn test_input(c: &mut Criterion, db: &mut Database, name: &'static str, input: impl AsRef<str>) {
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = db.search_and_rank_json(black_box(input.as_ref().to_owned()), 250);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_millis(350))
        .measurement_time(std::time::Duration::from_millis(1500));
    targets = bench_search
}
criterion_main!(benches);
