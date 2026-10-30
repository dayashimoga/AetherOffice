use crate::error::CopilotError;
use crate::tokenizer::{
    LocalTokenizer, CLS_TOKEN_ID, PAD_TOKEN_ID, SEP_TOKEN_ID, TOTAL_VOCAB_SIZE,
};
use serde::{Deserialize, Serialize};

pub const EMBEDDING_DIM: usize = 384;

/// 8-bit quantized vector for high-performance compact vector representation and SIMD dot products.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantizedVector {
    pub data: Vec<i8>,
    pub scale: f32,
}

impl QuantizedVector {
    /// Quantize a floating-point vector into int8 representation.
    pub fn from_f32(vec: &[f32]) -> Self {
        let mut max_abs: f32 = 0.0;
        for &val in vec {
            let abs = val.abs();
            if abs > max_abs {
                max_abs = abs;
            }
        }

        if max_abs == 0.0 {
            return Self {
                data: vec![0; vec.len()],
                scale: 1.0,
            };
        }

        let scale = max_abs / 127.0;
        let inv_scale = 1.0 / scale;
        let mut data = Vec::with_capacity(vec.len());

        for &val in vec {
            let scaled = (val * inv_scale).round();
            let clamped = scaled.clamp(-127.0, 127.0) as i8;
            data.push(clamped);
        }

        Self { data, scale }
    }

    /// Dequantize back to standard f32 representation.
    pub fn to_f32(&self) -> Vec<f32> {
        self.data.iter().map(|&q| (q as f32) * self.scale).collect()
    }

    /// Fast integer dot product with scale factor adjustment.
    pub fn dot_product(&self, other: &QuantizedVector) -> Result<f32, CopilotError> {
        if self.data.len() != other.data.len() {
            return Err(CopilotError::DimensionMismatch {
                expected: self.data.len(),
                found: other.data.len(),
            });
        }

        let mut acc: i32 = 0;
        let len = self.data.len();
        let chunks = len / 8;

        for chunk_idx in 0..chunks {
            let offset = chunk_idx * 8;
            acc += (self.data[offset] as i32) * (other.data[offset] as i32);
            acc += (self.data[offset + 1] as i32) * (other.data[offset + 1] as i32);
            acc += (self.data[offset + 2] as i32) * (other.data[offset + 2] as i32);
            acc += (self.data[offset + 3] as i32) * (other.data[offset + 3] as i32);
            acc += (self.data[offset + 4] as i32) * (other.data[offset + 4] as i32);
            acc += (self.data[offset + 5] as i32) * (other.data[offset + 5] as i32);
            acc += (self.data[offset + 6] as i32) * (other.data[offset + 6] as i32);
            acc += (self.data[offset + 7] as i32) * (other.data[offset + 7] as i32);
        }

        for i in (chunks * 8)..len {
            acc += (self.data[i] as i32) * (other.data[i] as i32);
        }

        Ok((acc as f32) * self.scale * other.scale)
    }
}

/// Computes the cosine similarity between two normalized f32 vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, CopilotError> {
    if a.len() != b.len() {
        return Err(CopilotError::DimensionMismatch {
            expected: a.len(),
            found: b.len(),
        });
    }

    let mut dot: f32 = 0.0;
    let len = a.len();
    let chunks = len / 8;

    for chunk_idx in 0..chunks {
        let offset = chunk_idx * 8;
        dot += a[offset] * b[offset];
        dot += a[offset + 1] * b[offset + 1];
        dot += a[offset + 2] * b[offset + 2];
        dot += a[offset + 3] * b[offset + 3];
        dot += a[offset + 4] * b[offset + 4];
        dot += a[offset + 5] * b[offset + 5];
        dot += a[offset + 6] * b[offset + 6];
        dot += a[offset + 7] * b[offset + 7];
    }

    for i in (chunks * 8)..len {
        dot += a[i] * b[i];
    }

    // Clamp for float precision
    Ok(dot.clamp(-1.0, 1.0))
}

/// Computes cosine similarity between two quantized vectors.
pub fn cosine_similarity_quantized(
    a: &QuantizedVector,
    b: &QuantizedVector,
) -> Result<f32, CopilotError> {
    let dot = a.dot_product(b)?;
    Ok(dot.clamp(-1.0, 1.0))
}

/// Zero-telemetry deterministic local embedding generator.
#[derive(Debug, Clone)]
pub struct LocalEmbeddingGenerator {
    tokenizer: LocalTokenizer,
    /// Projection weights: TOTAL_VOCAB_SIZE * EMBEDDING_DIM
    projection_weights: Vec<f32>,
    /// Positional encoding table for up to 512 tokens: 512 * EMBEDDING_DIM
    position_table: Vec<f32>,
}

impl Default for LocalEmbeddingGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalEmbeddingGenerator {
    /// Initializes deterministic weights without network dependency or external model files.
    pub fn new() -> Self {
        let tokenizer = LocalTokenizer::new();
        let total_weights = TOTAL_VOCAB_SIZE * EMBEDDING_DIM;
        let mut projection_weights = vec![0.0f32; total_weights];

        // Deterministic SplitMix64 PRNG
        let mut state: u64 = 0x9e3779b97f4a7c15;
        let next_splitmix = |s: &mut u64| -> f32 {
            *s = s.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = *s;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^= z >> 31;
            ((z as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32
        };

        // 8 Topic Clusters
        const NUM_TOPICS: usize = 8;
        let mut topic_centroids = vec![vec![0.0f32; EMBEDDING_DIM]; NUM_TOPICS];
        for centroid in topic_centroids.iter_mut() {
            for val in centroid.iter_mut() {
                *val = next_splitmix(&mut state);
            }
            Self::normalize_l2(centroid);
        }

        let finance_words = [
            "revenue",
            "profit",
            "loss",
            "ebitda",
            "earnings",
            "income",
            "expense",
            "fiscal",
            "quarter",
            "q1",
            "q2",
            "q3",
            "q4",
            "budget",
            "forecast",
            "sales",
            "cost",
            "margin",
            "balance",
            "cash",
            "flow",
            "audit",
            "tax",
            "roi",
            "equity",
            "liability",
            "asset",
            "dollar",
            "currency",
            "share",
            "financial",
            "expenditure",
        ];
        let calc_words = [
            "sheet",
            "spreadsheet",
            "workbook",
            "formula",
            "sum",
            "average",
            "count",
            "min",
            "max",
            "if",
            "vlookup",
            "hlookup",
            "xlookup",
            "index",
            "match",
            "filter",
            "sort",
            "pivot",
            "pivottable",
            "cell",
            "range",
            "coordinate",
            "grid",
            "calculate",
        ];
        let doc_words = [
            "document",
            "paragraph",
            "heading",
            "section",
            "page",
            "title",
            "text",
            "style",
            "font",
            "size",
            "color",
            "margin",
            "header",
            "footer",
            "table",
            "border",
            "align",
            "center",
            "left",
            "right",
            "justify",
            "bold",
            "italic",
            "underline",
            "bullet",
            "list",
            "word",
            "writer",
        ];
        let pres_words = [
            "presentation",
            "slide",
            "slides",
            "master",
            "layout",
            "shape",
            "rect",
            "ellipse",
            "triangle",
            "arrow",
            "bezier",
            "path",
            "transition",
            "animation",
            "timeline",
            "duration",
            "delay",
            "keyframe",
            "morph",
            "presenter",
            "speaker",
            "notes",
            "laser",
            "pointer",
            "powerpoint",
        ];
        let strategy_words = [
            "strategy",
            "executive",
            "vision",
            "roadmap",
            "leadership",
            "market",
            "customer",
            "product",
            "growth",
            "objective",
            "goal",
            "kpi",
            "target",
            "variance",
            "actual",
            "estimate",
            "review",
            "analysis",
            "contract",
            "agreement",
            "corporate",
        ];
        let tech_words = [
            "software",
            "rust",
            "microservices",
            "cloud",
            "server",
            "code",
            "infrastructure",
            "data",
            "network",
            "engine",
            "runtime",
            "api",
            "system",
            "memory",
            "wasm",
        ];

        // Generate per-token projection vector
        let matches_topic = |decoded: &str, words: &[&str]| -> bool {
            if decoded.is_empty() {
                return false;
            }
            for &w in words {
                if decoded == w {
                    return true;
                }
                if decoded.len() >= 4
                    && w.len() >= 4
                    && (decoded.starts_with(w) || w.starts_with(decoded))
                {
                    return true;
                }
            }
            false
        };

        for token_id in 0..(TOTAL_VOCAB_SIZE as u32) {
            let offset = (token_id as usize) * EMBEDDING_DIM;
            let decoded_raw = tokenizer.decode(&[token_id]);
            let decoded = decoded_raw.trim().to_lowercase();

            let mut token_vec = vec![0.0f32; EMBEDDING_DIM];

            // Check topic affinity
            let mut matched_topic = None;
            if matches_topic(&decoded, &finance_words) {
                matched_topic = Some(0);
            } else if matches_topic(&decoded, &calc_words) {
                matched_topic = Some(1);
            } else if matches_topic(&decoded, &doc_words) {
                matched_topic = Some(2);
            } else if matches_topic(&decoded, &pres_words) {
                matched_topic = Some(3);
            } else if matches_topic(&decoded, &strategy_words) {
                matched_topic = Some(4);
            } else if matches_topic(&decoded, &tech_words) {
                matched_topic = Some(5);
            }

            if let Some(t_idx) = matched_topic {
                for d in 0..EMBEDDING_DIM {
                    token_vec[d] += topic_centroids[t_idx][d] * 3.0;
                }
            }

            // Word-specific unique perturbation
            for val in token_vec.iter_mut() {
                *val += next_splitmix(&mut state) * 0.5;
            }

            Self::normalize_l2(&mut token_vec);

            projection_weights[offset..(offset + EMBEDDING_DIM)]
                .copy_from_slice(&token_vec[..EMBEDDING_DIM]);
        }

        // Positional encoding table (max seq len 512)
        let max_seq_len = 512;
        let mut position_table = Vec::with_capacity(max_seq_len * EMBEDDING_DIM);
        for pos in 0..max_seq_len {
            for i in 0..EMBEDDING_DIM {
                let div_term = ((i / 2) as f32 * 2.0 / EMBEDDING_DIM as f32).exp2();
                let angle =
                    (pos as f32) / (10000.0f32.powf(div_term / EMBEDDING_DIM as f32).max(1.0));
                let val = if i % 2 == 0 { angle.sin() } else { angle.cos() };
                position_table.push(val * 0.01); // Scaled positional contribution
            }
        }

        Self {
            tokenizer,
            projection_weights,
            position_table,
        }
    }

    /// Access the internal tokenizer.
    pub fn tokenizer(&self) -> &LocalTokenizer {
        &self.tokenizer
    }

    /// Normalizes a vector in-place to unit length (L2 norm = 1.0).
    fn normalize_l2(vec: &mut [f32]) {
        let mut sum_sq: f32 = 0.0;
        for &v in vec.iter() {
            sum_sq += v * v;
        }

        if sum_sq > 0.0 {
            let inv_norm = 1.0 / sum_sq.sqrt();
            for v in vec.iter_mut() {
                *v *= inv_norm;
            }
        } else {
            let uniform = 1.0 / (vec.len() as f32).sqrt();
            for v in vec.iter_mut() {
                *v = uniform;
            }
        }
    }

    /// Embeds a token ID sequence into a normalized 384-dimensional vector.
    pub fn embed_token_ids(&self, token_ids: &[u32]) -> Vec<f32> {
        let mut vector = vec![0.0f32; EMBEDDING_DIM];
        if token_ids.is_empty() {
            Self::normalize_l2(&mut vector);
            return vector;
        }

        let mut total_weight = 0.0f32;

        for (pos, &token_id) in token_ids.iter().enumerate() {
            if token_id == PAD_TOKEN_ID {
                continue;
            }

            // Down-weight special bracket tokens [CLS] and [SEP]
            let token_weight = if token_id == CLS_TOKEN_ID || token_id == SEP_TOKEN_ID {
                0.3f32
            } else {
                1.0f32
            };

            let vocab_idx = (token_id as usize) % TOTAL_VOCAB_SIZE;
            let weight_offset = vocab_idx * EMBEDDING_DIM;
            let pos_offset = (pos.min(511)) * EMBEDDING_DIM;

            for (d, v) in vector.iter_mut().enumerate() {
                let w = self.projection_weights[weight_offset + d];
                let p = self.position_table[pos_offset + d];
                *v += (w + p) * token_weight;
            }

            total_weight += token_weight;
        }

        if total_weight > 0.0 {
            let inv_weight = 1.0 / total_weight;
            for v in vector.iter_mut() {
                *v *= inv_weight;
            }
        }

        Self::normalize_l2(&mut vector);
        vector
    }

    /// Generate normalized embedding vector from string text.
    pub fn embed_text(&self, text: &str) -> Vec<f32> {
        let tokens = self.tokenizer.tokenize(text);
        self.embed_token_ids(&tokens)
    }

    /// Generate quantized int8 vector from string text.
    pub fn embed_quantized(&self, text: &str) -> QuantizedVector {
        let f32_vec = self.embed_text(text);
        QuantizedVector::from_f32(&f32_vec)
    }
}
