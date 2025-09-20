use ollama_rs::{
    Ollama,
    generation::{
        embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest},
        parameters::{KeepAlive, TimeUnit},
    },
};

pub struct OllamaEmbedder {
    ollama: Ollama,
}

impl OllamaEmbedder {
    pub fn new() -> OllamaEmbedder {
        Self {
            ollama: Ollama::default(),
        }
    }
    pub async fn generate(&self, text: &String) -> Option<Vec<Vec<f32>>> {
        let request = GenerateEmbeddingsRequest::new(
            "nomic-embed-text:v1.5".to_owned(),
            EmbeddingsInput::Multiple((vec![text.to_string()])),
        )
        .keep_alive(KeepAlive::Until {
            time: 1,
            unit: TimeUnit::Hours,
        });

        let response = self.ollama.generate_embeddings(request).await;

        if let Ok(embed) = response {
            Some(embed.embeddings)
        } else {
            None
        }
    }
    pub fn generate_seqentially(&self, text: &String) -> Option<Vec<Vec<f32>>> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.generate(text))
    }
    pub fn comparare_cos(embed_1: &[f32], embed_2: &[f32]) -> f32 {
        assert!(embed_1.len() == embed_2.len());
        let n = embed_1.len().min(embed_2.len());
        if n == 0 {
            return 0.0;
        }

        let mut dot: f32 = 0.0;
        let mut sum_sq_a: f32 = 0.0;
        let mut sum_sq_b: f32 = 0.0;

        for i in 0..n {
            let a = embed_1[i];
            let b = embed_2[i];
            dot += a * b;
            sum_sq_a += a * a;
            sum_sq_b += b * b;
        }

        if sum_sq_a <= 0.0 || sum_sq_b <= 0.0 {
            0.0
        } else {
            dot / (sum_sq_a.sqrt() * sum_sq_b.sqrt())
        }
    }
    pub fn down_project(vector: &[f32], new_len: usize) -> Vec<f32> {
        let old_len = vector.len();
        assert!(new_len > 0 && new_len <= old_len);

        let mut result = Vec::with_capacity(new_len);
        let chunk_size = old_len as f32 / new_len as f32;

        for i in 0..new_len {
            let start = (i as f32 * chunk_size).floor() as usize;
            let end = (((i + 1) as f32 * chunk_size).ceil() as usize).min(old_len);

            if start < end {
                let sum: f32 = vector[start..end].iter().sum();
                let avg = sum / (end - start) as f32;
                result.push(avg);
            } else {
                result.push(0.0);
            }
        }
        result
    }
}
