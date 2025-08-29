use criterion::{criterion_group, criterion_main, Criterion, black_box};
use backend::{data::{Journey2025MainPage, ScrapedMainPageEnum, Summer2025MainPage}, database::Database};

fn bench_add_entry(c: &mut Criterion) {
    let db = Database::new_non_backed();
    let entry = ScrapedMainPageEnum::Journey2025(Journey2025MainPage::default());
    c.bench_function("add_entry", |b| {
        b.iter(|| {
            db.add_entry(black_box(entry.clone()));
        })
    });
}

fn bench_search(c: &mut Criterion) {
    let db = Database::load_file("/Users/ryan/Github/searxing-hc/complete_database.json");

    c.bench_function("search_5_words", |b| {
        b.iter(|| {
            let _ = db.search_and_rank_json(black_box("robot simulator sand ai iot".to_string()), 100);
        })
    });

    c.bench_function("search_overhead", |b| {
        b.iter(|| {
            let _ = db.search_and_rank_json(black_box("".to_string()), 1);
        })
    });

    c.bench_function("search_json_overhead", |b| {
        b.iter(|| {
            let _ = db.search_and_rank_json(black_box("".to_string()), 1000);
        })
    });
}






criterion_group!(benches, bench_add_entry, bench_search);
criterion_main!(benches);