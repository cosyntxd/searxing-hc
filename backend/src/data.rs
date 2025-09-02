use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Debug)]
pub struct DetailedSearchResult {
    pub id: usize,
    pub rank: f32,
    pub event: String,
    // generated per event
    pub page: GenericPreviewSearchData,
}

#[derive(Serialize, Debug)]
pub struct GenericPreviewSearchData {
    pub img: String,
    pub name: String,
    pub description: String,
    pub props: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComputedData {
    #[serde(with = "BigArray")]
    pub embedding: [f32; 768],
    pub ai_description: f32,
    pub ai_code: f32,
}
#[derive(Eq, Hash, PartialEq)]
pub struct UniqueString(pub String);

#[enum_dispatch]
pub trait DatabasePage {
    fn preview(&self) -> GenericPreviewSearchData;
    fn unique_string(&self) -> UniqueString;
    fn rank(&self, query: &String) -> f32;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[enum_dispatch(DatabasePage)]
pub enum ScrapedMainPageEnum {
    Journey2025(Journey2025MainPage),
    Summer2025(Summer2025MainPage),
}

// Journey 2025
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Journey2025MainPage {
    pub id: u32,
    pub main_image: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub followers: u16,
    pub stonks: u16,
    pub time: String,
    pub readme: Option<String>,
    pub repo: Option<String>,
    pub demo: Option<String>,
    pub updates: Vec<Journey2025IndividualUpdate>,
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Journey2025IndividualUpdate {
    time: String,
    message: String,
    attatchments: Vec<String>,
}
impl DatabasePage for Journey2025MainPage {
    fn rank(&self, query: &String) -> f32 {
        self.followers as f32 + self.stonks as f32 * 0.2
    }

    fn unique_string(&self) -> UniqueString {
        UniqueString(format!("{}", self.id))
    }
    fn preview(&self) -> GenericPreviewSearchData {
        todo!()
    }
}

// Summer of Making 2025
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Summer2025MainPage {
    pub url: String,
    pub main_image: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub followers: u16,
    pub time: u32,
    pub readme: Option<String>,
    pub repo: Option<String>,
    pub demo: Option<String>,
    pub updates: Vec<Summer2025IndividualUpdate>,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Summer2025IndividualUpdate {
    pub time: u32,
    pub message: String,
    pub image: Option<String>,
}
impl DatabasePage for Summer2025MainPage {
    fn rank(&self, query: &String) -> f32 {
        let query = query.to_lowercase();
        let mut rank = 0.0;

        for (i, word) in query.replace('-', " ").split_whitespace().enumerate() {
            let word_scale = 0.45 + 0.55 * (-0.2231 * i as f32).exp();

            rank += self
                .name
                .to_lowercase()
                .split_whitespace()
                .filter(|s| s.contains(word))
                .count() as f32
                * 5.11
                * word_scale;

            rank += self
                .description
                .to_lowercase()
                .split_whitespace()
                .filter(|s| s.contains(word))
                .count() as f32
                * 3.27
                * word_scale;
        }
        // yes
        if rank > 1.0 {
            for word in query.replace('-', " ").split_whitespace() {
                for devlog in self.updates.clone() {
                    if devlog.message.to_lowercase().contains(word) {
                        rank += 0.9652;
                    }
                }
            }
        }
        if rank < 1.0 {
            rank -= 50.0;
        }
        if self.time < 1000 {
            rank -= 1.0;
        }
        if self.description.contains('—') {
            // em dash
            rank -= 0.8;
        }
        rank += (self.description.len() as f32 / 90.0).sqrt().min(1.2);
        rank += (self.updates.len() as f32 / 9.0).sqrt().min(2.3);
        rank += (self.followers as f32 / 4.5).min(4.3);
        rank += (self.time as f32 / 8_000.0).min(4.6);

        if self.demo.is_some() {
            rank += 0.85;
        }

        rank
    }

    fn unique_string(&self) -> UniqueString {
        UniqueString(self.url.clone())
    }
    fn preview(&self) -> GenericPreviewSearchData {
        GenericPreviewSearchData {
            img: self.main_image.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            props: format!("updates: {}", self.updates.len()),
        }
    }
}
