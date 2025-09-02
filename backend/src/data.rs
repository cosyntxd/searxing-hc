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
    fn rank(&self, query: &String, extra: &Option<ComputedData>) -> f32;
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
    fn preview(&self) -> GenericPreviewSearchData {
        todo!()
    }
    fn unique_string(&self) -> UniqueString {
        UniqueString(format!("{}", self.id))
    }
    fn rank(&self, query: &String, extra: &Option<ComputedData>) -> f32 {
        self.followers as f32 + self.stonks as f32 * 0.2
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
    fn preview(&self) -> GenericPreviewSearchData {
        GenericPreviewSearchData {
            img: self.main_image.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            props: format!("updates: {}", self.updates.len()),
        }
    }
    fn unique_string(&self) -> UniqueString {
        UniqueString(self.url.clone())
    }
    fn rank(&self, query: &String, extra: &Option<ComputedData>) -> f32 {
        // let mut acc = 0.0;
        // if let Some(val) = std::hint::black_box(extra) {
        //     for i in 0..768 {
        //         acc += val.embedding[i];
        //     }
        //     return  acc;
        // }
        // 0.0
        return self.description.split_ascii_whitespace().filter(|x| x == query).count() as f32 + self.time as f32;
    }
}
