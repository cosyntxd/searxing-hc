use std::fs;

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use backend::{data::{ScrapedMainPageEnum, Summer2025MainPage}, database::Database};

fn bench_database_new(c: &mut Criterion) {
    c.bench_function("Database::new_non_backed", |b| {
        b.iter(|| {
            let db = Database::new_non_backed();
            black_box(db);
        });
    });
}

fn bench_add_entry(c: &mut Criterion) {
    let word_list: Vec<String> = fs::read_to_string("../data/word_list.txt")
        .expect("failed to read word list")
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    fastrand::seed(0);

    let fake_phrase = |words: usize| -> String {
        (0..words)
            .map(|_| fastrand::choice(&word_list).unwrap().clone())
            .collect::<Vec<_>>()
            .join(" ")
    };


    let db = Database::new_non_backed();
    let dummy_entry = ScrapedMainPageEnum::Summer2025(Summer2025MainPage {
        url: "https://link".into(),
        main_image: "https://link".into(),
        name: fake_phrase(8),
        description: fake_phrase(120),
        author: "bob".into(),
        followers: 12,
        time: 12_000,
        readme: None,
        repo: None,
        demo: None,
        updates: vec![],
    });

    c.bench_function("Database::add_entry", |b| {
        b.iter(|| {
            db.add_entry(black_box(dummy_entry.clone()));
        });
    });
}

criterion_group!(
    benches,
    bench_database_new,
    bench_add_entry,
);
criterion_main!(benches);
