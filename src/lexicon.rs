use ::rand::seq::SliceRandom;
use ::rand::Rng;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, Debug, Default)]
pub struct SemanticValence {
    pub curiosity: f32,
    pub vitality: f32,
}

impl SemanticValence {
    pub const fn new(curiosity: f32, vitality: f32) -> Self {
        Self { curiosity, vitality }
    }

    pub fn dominant_mood_name(&self) -> &'static str {
        if self.curiosity > self.vitality { "Inquisitive" } else { "Bioluminescent" }
    }
}

pub struct LexiconEngine {
    chains: HashMap<(String, String), Vec<String>>,
    single_chains: HashMap<String, Vec<String>>,
    vocabulary: Vec<String>,
    pub harvested_stream: VecDeque<String>,
    pub live_composition: String,
}

impl LexiconEngine {
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
            single_chains: HashMap::new(),
            vocabulary: Vec::new(),
            harvested_stream: VecDeque::with_capacity(24),
            live_composition: "The first postcephalic appendage is modified into resonant".to_string(),
        }
    }

    pub fn ingest_corpus(&mut self, text: &str) {
        for raw_sentence in text.split(|c| c == '.' || c == '!' || c == '?' || c == '\n') {
            let tokens: Vec<String> = raw_sentence
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string())
                .filter(|w| w.len() >= 2)
                .collect();

            for t in &tokens {
                if t.len() >= 4 && self.vocabulary.len() < 2500 && !self.vocabulary.contains(t) {
                    self.vocabulary.push(t.clone());
                }
            }

            if tokens.len() < 3 { continue; }

            for i in 0..tokens.len() - 1 {
                self.single_chains.entry(tokens[i].to_lowercase()).or_default().push(tokens[i + 1].clone());
            }

            for window in tokens.windows(3) {
                let key = (window[0].to_lowercase(), window[1].to_lowercase());
                self.chains.entry(key).or_default().push(window[2].clone());
            }
        }
    }

    pub fn mutate_word_in_context<R: Rng>(&self, rng: &mut R, prev_prev: Option<&str>, prev: Option<&str>, current: &str) -> String {
        let clean_prev = prev.map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_lowercase());
        let clean_pp = prev_prev.map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_lowercase());

        if let (Some(pp), Some(p)) = (clean_pp, clean_prev.clone()) {
            if let Some(opts) = self.chains.get(&(pp, p)) {
                if let Some(chosen) = opts.choose(rng) { return match_punctuation(current, chosen); }
            }
        }

        if let Some(p) = clean_prev {
            if let Some(opts) = self.single_chains.get(&p) {
                if let Some(chosen) = opts.choose(rng) { return match_punctuation(current, chosen); }
            }
        }

        if let Some(chosen) = self.vocabulary.choose(rng) {
            return match_punctuation(current, chosen);
        }

        current.to_string()
    }

    pub fn absorb_plucked_word<R: Rng>(&mut self, rng: &mut R, word: &str) {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string();
        if clean.is_empty() { return; }

        self.harvested_stream.push_back(clean.clone());
        if self.harvested_stream.len() > 18 { self.harvested_stream.pop_front(); }

        let mut current_words: Vec<String> = self.live_composition.split_whitespace().map(|s| s.to_string()).collect();
        if current_words.len() >= 13 { current_words.drain(0..4); }

        if let Some(last) = current_words.last().cloned() {
            let key = last.to_lowercase();
            if let Some(next_opts) = self.single_chains.get(&key) {
                if let Some(bridge) = next_opts.choose(rng) {
                    if bridge.len() <= 4 { current_words.push(bridge.clone()); }
                }
            }
        }

        current_words.push(clean);
        let mut rebuilt = current_words.join(" ");
        if let Some(first) = rebuilt.get_mut(0..1) { first.make_ascii_uppercase(); }
        self.live_composition = rebuilt;
    }
}

fn match_punctuation(original: &str, replacement: &str) -> String {
    let mut out = replacement.to_string();
    if original.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        if let Some(first) = out.get_mut(0..1) { first.make_ascii_uppercase(); }
    }
    if original.ends_with(',') { out.push(','); }
    else if original.ends_with('.') { out.push('.'); }
    else if original.ends_with(';') { out.push(';'); }
    else if original.ends_with(')') { out.push(')'); }
    out
}
