use std::collections::HashMap;

pub const PAD_TOKEN_ID: u32 = 0;
pub const UNK_TOKEN_ID: u32 = 1;
pub const CLS_TOKEN_ID: u32 = 2;
pub const SEP_TOKEN_ID: u32 = 3;
pub const MASK_TOKEN_ID: u32 = 4;

/// Base reserved vocabulary size.
pub const BASE_VOCAB_SIZE: usize = 1024;
/// Hash bucket size for out-of-vocabulary subwords.
pub const OOV_BUCKET_SIZE: usize = 1024;
/// Total effective vocabulary space.
pub const TOTAL_VOCAB_SIZE: usize = BASE_VOCAB_SIZE + OOV_BUCKET_SIZE;

/// Lightweight, deterministic on-device subword tokenizer.
#[derive(Debug, Clone)]
pub struct LocalTokenizer {
    token_to_id: HashMap<String, u32>,
    id_to_token: HashMap<u32, String>,
}

impl Default for LocalTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalTokenizer {
    pub fn new() -> Self {
        let mut token_to_id = HashMap::with_capacity(BASE_VOCAB_SIZE);
        let mut id_to_token = HashMap::with_capacity(BASE_VOCAB_SIZE);

        let special_tokens = [
            ("[PAD]", PAD_TOKEN_ID),
            ("[UNK]", UNK_TOKEN_ID),
            ("[CLS]", CLS_TOKEN_ID),
            ("[SEP]", SEP_TOKEN_ID),
            ("[MASK]", MASK_TOKEN_ID),
        ];

        for (token, id) in special_tokens {
            token_to_id.insert(token.to_string(), id);
            id_to_token.insert(id, token.to_string());
        }

        let mut next_id = 5u32;

        // Common single characters (ASCII 32 to 126)
        for b in 32u8..=126u8 {
            let c = b as char;
            let s = c.to_string();
            if !token_to_id.contains_key(&s) {
                token_to_id.insert(s.clone(), next_id);
                id_to_token.insert(next_id, s);
                next_id += 1;
            }
        }

        // Domain keywords for Office documents, spreadsheets, slides, formulas, finances
        let domain_words = [
            "the",
            "of",
            "and",
            "to",
            "a",
            "in",
            "for",
            "is",
            "on",
            "that",
            "by",
            "this",
            "with",
            "i",
            "you",
            "it",
            "not",
            "or",
            "be",
            "are",
            "from",
            "at",
            "as",
            "your",
            "all",
            "have",
            "new",
            "more",
            "an",
            "was",
            "we",
            "will",
            "home",
            "can",
            "us",
            "about",
            "if",
            "page",
            "my",
            "has",
            "search",
            "free",
            "but",
            "our",
            "one",
            "other",
            "do",
            "no",
            "information",
            "time",
            "they",
            "site",
            "he",
            "up",
            "may",
            "what",
            "which",
            "their",
            "news",
            "out",
            "use",
            "any",
            "there",
            "see",
            "only",
            "so",
            "his",
            "when",
            "contact",
            "here",
            "business",
            // Office & Document concepts
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
            "row",
            "column",
            "cell",
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
            "numbered",
            "list",
            "indent",
            "spacing",
            "break",
            "line",
            "character",
            "run",
            "bookmark",
            // Spreadsheet concepts
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
            "chart",
            "data",
            "range",
            "reference",
            "spill",
            "array",
            "calc",
            "calculate",
            "grid",
            "coordinate",
            "value",
            "numeric",
            "string",
            "boolean",
            "error",
            "blank",
            "currency",
            "percentage",
            "date",
            "time",
            "format",
            "conditional",
            "freeze",
            "pane",
            "autofill",
            // Presentation concepts
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
            "notespane",
            "filmstrip",
            "laser",
            "pointer",
            "ink",
            "pen",
            "annotation",
            "studio",
            "fullscreen",
            "f5",
            "media",
            "video",
            "audio",
            "embedded",
            "geometry",
            "transform",
            "matrix",
            "rotation",
            "scale",
            // Financial & Business terminology
            "revenue",
            "profit",
            "loss",
            "ebitda",
            "margin",
            "income",
            "expense",
            "budget",
            "forecast",
            "fiscal",
            "quarter",
            "quarterly",
            "q1",
            "q2",
            "q3",
            "q4",
            "annual",
            "year",
            "growth",
            "cost",
            "sales",
            "asset",
            "liability",
            "equity",
            "cash",
            "flow",
            "balance",
            "audit",
            "tax",
            "roi",
            "kpi",
            "target",
            "variance",
            "actual",
            "estimate",
            "strategy",
            "executive",
            "summary",
            "report",
            "review",
            "analysis",
            "market",
            "customer",
            "product",
            "service",
            "contract",
            "agreement",
            "financial",
            "finance",
            "earnings",
            "earning",
            "expenditure",
            "statement",
            "operating",
            "subscription",
            "capital",
            "shares",
            "share",
            "stock",
            "accounting",
            // Subwords and affixes
            "##ing",
            "##ed",
            "##tion",
            "##sion",
            "##ly",
            "##al",
            "##ment",
            "##able",
            "##ible",
            "##ness",
            "##less",
            "##ful",
            "##ive",
            "##ous",
            "##ize",
            "##ise",
            "##ate",
            "##ity",
            "##ty",
            "##est",
            "##er",
            "##or",
            "##ic",
            "##ism",
            "##ist",
            "##ize",
            "##ary",
            "##ry",
            "re##",
            "un##",
            "in##",
            "im##",
            "dis##",
            "mis##",
            "pre##",
            "post##",
            "sub##",
            "inter##",
            "trans##",
            "over##",
            "under##",
            "multi##",
            "non##",
            "auto##",
            "semi##",
            "anti##",
        ];

        for word in domain_words {
            if !token_to_id.contains_key(word) && next_id < BASE_VOCAB_SIZE as u32 {
                token_to_id.insert(word.to_string(), next_id);
                id_to_token.insert(next_id, word.to_string());
                next_id += 1;
            }
        }

        Self {
            token_to_id,
            id_to_token,
        }
    }

    /// Number of distinct tokens in the base vocabulary.
    pub fn vocab_size(&self) -> usize {
        TOTAL_VOCAB_SIZE
    }

    /// Deterministic 32-bit FNV-1a hash folded into out-of-vocabulary bucket space.
    fn hash_oov(token: &str) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        for byte in token.as_bytes() {
            hash ^= *byte as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        BASE_VOCAB_SIZE as u32 + (hash % OOV_BUCKET_SIZE as u32)
    }

    /// Tokenizes an individual word using longest-prefix subword matching.
    pub fn tokenize_word(&self, word: &str) -> Vec<u32> {
        let lower = word.to_lowercase();
        if lower.is_empty() {
            return Vec::new();
        }

        // Direct match
        if let Some(&id) = self.token_to_id.get(&lower) {
            return vec![id];
        }

        let mut tokens = Vec::new();
        let chars: Vec<char> = lower.chars().collect();
        let len = chars.len();
        let mut start = 0;

        while start < len {
            let mut end = len;
            let mut matched = false;

            while end > start {
                let sub: String = chars[start..end].iter().collect();
                let candidate = if start == 0 {
                    sub.clone()
                } else {
                    format!("##{}", sub)
                };

                if let Some(&id) = self.token_to_id.get(&candidate) {
                    tokens.push(id);
                    matched = true;
                    start = end;
                    break;
                }
                end -= 1;
            }

            if !matched {
                // If subword not matched, hash the remaining token as a whole unit
                let remaining: String = chars[start..].iter().collect();
                tokens.push(Self::hash_oov(&remaining));
                break;
            }
        }

        tokens
    }

    /// Full text tokenizer returning sequence of token IDs bracketed by [CLS] and [SEP].
    pub fn tokenize(&self, text: &str) -> Vec<u32> {
        let mut tokens = vec![CLS_TOKEN_ID];

        for word in text.split_whitespace() {
            let mut current_word = String::new();
            for c in word.chars() {
                if c.is_alphanumeric() {
                    current_word.push(c);
                } else {
                    if !current_word.is_empty() {
                        tokens.extend(self.tokenize_word(&current_word));
                        current_word.clear();
                    }
                    let punct = c.to_string();
                    if let Some(&id) = self.token_to_id.get(&punct) {
                        tokens.push(id);
                    } else {
                        tokens.push(Self::hash_oov(&punct));
                    }
                }
            }
            if !current_word.is_empty() {
                tokens.extend(self.tokenize_word(&current_word));
            }
        }

        tokens.push(SEP_TOKEN_ID);
        tokens
    }

    /// Decodes a slice of token IDs back into string format.
    pub fn decode(&self, ids: &[u32]) -> String {
        let mut words = Vec::new();
        for &id in ids {
            if id == PAD_TOKEN_ID || id == CLS_TOKEN_ID || id == SEP_TOKEN_ID {
                continue;
            }
            if let Some(token) = self.id_to_token.get(&id) {
                if token.starts_with("##") {
                    if let Some(last) = words.last_mut() {
                        let suffix = token.trim_start_matches("##");
                        let mut combined = String::from(&*last);
                        combined.push_str(suffix);
                        *last = combined;
                    } else {
                        words.push(token.trim_start_matches("##").to_string());
                    }
                } else {
                    words.push(token.clone());
                }
            } else {
                words.push(format!("[ID:{}]", id));
            }
        }
        words.join(" ")
    }
}
