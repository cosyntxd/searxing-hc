use std::{cmp::Reverse, collections::{BinaryHeap, HashMap}, fs::{self, File}, io::Write, sync::RwLock, time::Instant};

use ordered_float::OrderedFloat;

use crate::data::{DatabasePage, DetailedSearchResult, ExtraData, ScrapedMainPageEnum, ScrapedPage};



pub struct Database {
    pub raw_data: RwLock<Vec<ScrapedPage>>,
    pub relational: HashMap<String, usize>,
    pub file_location: &'static str,
}

impl Database {
    pub fn new_non_backed() -> Database {
        Database {
            raw_data: RwLock::new(vec![]),
            relational: HashMap::new(),
            file_location: "",
        }
    }
    pub fn load_file(name: &'static str) -> Database {
        let raw_data_from_file = match fs::read_to_string(name) {
            Ok(data) => {
                match serde_json::from_str(&data) {
                    Ok(good) => good,
                    Err(e) => {
                        eprintln!("cant load from json");
                        None
                    },
                }
            },
            Err(e) => {
                eprintln!("cant load file");
                if let Err(f) = File::create(name) {
                    eprintln!("cant make new file (bad)");
                }
                None
            },
        };
        // why tf cant type be infered, lsp knows but not rustc 
        let raw_data: Vec<ScrapedPage> = raw_data_from_file.unwrap_or_default();

        let mut relational = HashMap::new();

        for (i, entry) in raw_data.iter().enumerate() {
            relational.insert(entry.page.unique_string(), i);
        }

        Database {
            raw_data: RwLock::new(raw_data),
            relational,
            file_location: name,
        }
    }
    pub fn save_json(&self) {
        let _guard = self.raw_data.write().unwrap();
        let data = &*_guard;
        let json_string = serde_json::to_string_pretty(data).unwrap();

        let mut file = fs::File::create(self.file_location).unwrap();
        file.write_all(json_string.as_bytes()).unwrap();
    }
    pub fn add_entry(&self, entry: ScrapedMainPageEnum) {
        let mut data = self.raw_data.write().unwrap();

        let page = ScrapedPage {
            page: entry.clone(),
            extra: ExtraData {
                embedding: [0.0; 768],
                score_multiplier: 1.0,
                embed_good: false,
            }
        };

        if let Some(existing_idx) = self.relational.get(&entry.unique_string()) {
            data[*existing_idx] = page;
        } else {
            data.push(page);
        }
    }
    pub fn search_and_rank_json(&self, query: String, k: usize) -> String {
        let data = self.raw_data.read().unwrap();
        let mut min_heap: BinaryHeap<Reverse<(OrderedFloat<f32>, usize)>> = BinaryHeap::with_capacity(50);

        for (i, page) in data.iter().enumerate() {
            let current_rank = OrderedFloat(page.page.rank(&query));
            if current_rank.0 < 0.0 && !query.is_empty(){
                continue;
            }
            let heap_item = Reverse((current_rank, i));

            if min_heap.len() < k {
                min_heap.push(heap_item);
            } else {   
                if current_rank > min_heap.peek().unwrap().0.0 {
                    min_heap.pop();
                    min_heap.push(heap_item);
                }
            }
        }
        let mut top_page_info: Vec<(OrderedFloat<f32>, usize)> = min_heap
            .into_iter()
            .map(|reverse_item| reverse_item.0)
            .collect();

        top_page_info.sort_by(|a, b| {
            b.0.cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
        });

        let top_pages = top_page_info
            .into_iter()
            .map(|(rank, original_index)| DetailedSearchResult {
                rank: rank.0,
                id: original_index,
                event: data[original_index].page.unique_string(), 
                page: data[original_index].page.preview(),
            })
            .collect::<Vec<DetailedSearchResult>>();

        serde_json::to_string_pretty(&top_pages).unwrap()
    }

    pub fn set_multiplier(&self, index: usize, score: f32) {
        let mut data = self.raw_data.write().unwrap();
        assert!(index < data.len());
        data[index].extra.score_multiplier = score;
    }
    pub fn set_embedding(&self, index: usize, embed: Vec<f32>) {
        let mut data = self.raw_data.write().unwrap();
        assert!(index < data.len());
        let target = & mut data[index].extra;
        assert!(target.embedding.len() == embed.len());
        target.embed_good = true;
        target.embedding = embed.try_into().unwrap();
    }
}
