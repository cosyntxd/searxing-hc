use backend::{
    data::{
        Journey2025MainPage, ScrapedMainPageEnum, Summer2025IndividualUpdate, Summer2025MainPage,
    },
    database::Database,
};
use criterion::{Criterion, criterion_group, criterion_main};
use itertools::Itertools;
use std::{fs, hint::black_box, path::Path};

const POPULAR_15: &'static str = "website game ai portfolio app ai bot for project tracker calculator discord learning python system";
const NICHE_15: &'static str = "fpga frc cad solder library ftc xrp SPY stock synthesizer sand simulator physics minecraft executable";
const ENG_15: &'static str = "a and the to for is with of in you that it this your on";

fn bench_search(c: &mut Criterion) {
    let db = &mut Database::new_non_backed();

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

    for _ in 0..10_000 {
        let updates = Summer2025IndividualUpdate {
            time: fastrand::u32(0..9_999),
            message: fake_phrase(60),
            image: None,
        };
        let summer_fake = Summer2025MainPage {
            url: format!(
                "https://hackclub.com/project/{}",
                fastrand::u32(0..99999999)
            ),
            main_image: format!(
                "https://hackclub.com/project/{}",
                fastrand::u32(0..99999999)
            ),
            name: fake_phrase(5),
            description: fake_phrase(150),
            author: format!("author-{}", fastrand::u32(0..99999999)),
            followers: fastrand::u16(0..99),
            time: fastrand::u32(0..99_999),
            readme: Some(format!("github.com/{}", fake_phrase(1))),
            repo: Some(format!("github.com/{}", fake_phrase(1))),
            demo: None,
            updates: vec![updates; fastrand::usize(0..10)],
        };
        db.add_entry(ScrapedMainPageEnum::Summer2025((summer_fake)));
    }

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
