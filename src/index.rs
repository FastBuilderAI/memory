// Ported from MahaBodi (github.com/mahabodi/mahabodi, crates/mahabodi-core/src/index.rs, MIT).
//! Inverted index over graph nodes: exact terms, stems, and a character-trigram
//! map over the vocabulary for typo-tolerant term lookup. Scoring is BM25.

use std::collections::{HashMap, HashSet};

use crate::graph::Graph;
use crate::text;

#[derive(Debug, Default, Clone)]
pub struct Index {
    exact: HashMap<String, Vec<(usize, f32)>>,
    stems: HashMap<String, Vec<(usize, f32)>>,
    doc_len: Vec<f32>,
    avg_len: f32,
    n_docs: usize,
    vocab: Vec<String>,
    tri: HashMap<String, Vec<usize>>,
}

/// Field weights: a term in a node's id/label says more than one in its passage.
pub const W_LABEL: f32 = 3.0;
pub const W_TEXT: f32 = 1.0;

/// Mean node length for BM25, accumulated in f64: an f32 running sum drifts once the total passes 2^24 (about 750K
/// nodes of ~22 terms), by 0.5 % and more depending on node order, which skews every BM25 length norm.
fn mean_len(doc_len: &[f32]) -> f32 {
    if doc_len.is_empty() { 0.0 } else { (doc_len.iter().map(|&x| x as f64).sum::<f64>() / doc_len.len() as f64) as f32 }
}

impl Index {
    pub fn build(g: &Graph) -> Index {
        let mut ix = Index { n_docs: g.nodes.len(), doc_len: vec![0.0; g.nodes.len()], ..Default::default() };
        // Postings are appended in node order, so every posting list is the same in every run.
        for (i, n) in g.nodes.iter().enumerate() {
            let mut tf: HashMap<String, f32> = HashMap::new();
            for t in text::terms(&n.label) {
                *tf.entry(t).or_default() += W_LABEL;
            }
            for t in text::terms(&n.text) {
                *tf.entry(t).or_default() += W_TEXT;
            }
            // whole-number weights: these f32 sums are exact in any order
            let len = tf.values().sum();
            let mut stf: HashMap<String, f32> = HashMap::new();
            for (t, w) in &tf {
                *stf.entry(text::stem(t)).or_default() += *w;
            }
            ix.doc_len[i] = len;
            for (t, w) in tf {
                ix.exact.entry(t).or_default().push((i, w));
            }
            for (t, w) in stf {
                ix.stems.entry(t).or_default().push((i, w));
            }
        }
        ix.avg_len = mean_len(&ix.doc_len);
        ix.vocab = ix.exact.keys().cloned().collect();
        ix.vocab.sort();
        for (vi, w) in ix.vocab.iter().enumerate() {
            for g in text::trigrams(w) {
                ix.tri.entry(g).or_default().push(vi);
            }
        }
        ix
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    pub fn has_term(&self, t: &str) -> bool {
        self.exact.contains_key(t)
    }

    pub fn doc_freq(&self, t: &str) -> usize {
        self.exact.get(t).map_or(0, |p| p.len())
    }

    fn bm25(&self, postings: &[(usize, f32)], weight: f32, scores: &mut HashMap<usize, f32>) {
        let (k1, b) = (1.2f32, 0.75f32);
        let df = postings.len() as f32;
        let idf = ((self.n_docs as f32 - df + 0.5) / (df + 0.5) + 1.0).ln();
        for &(d, tf) in postings {
            let norm = k1 * (1.0 - b + b * self.doc_len[d] / self.avg_len.max(1e-6));
            *scores.entry(d).or_default() += weight * idf * tf * (k1 + 1.0) / (tf + norm);
        }
    }

    /// BM25 over exact terms. Returns (node, score), unsorted.
    pub fn search_exact(&self, terms: &[String]) -> HashMap<usize, f32> {
        let mut s = HashMap::new();
        for t in dedup(terms) {
            if let Some(p) = self.exact.get(&t) {
                self.bm25(p, 1.0, &mut s);
            }
        }
        s
    }

    pub fn search_stem(&self, terms: &[String]) -> HashMap<usize, f32> {
        let mut s = HashMap::new();
        for t in dedup(terms) {
            if let Some(p) = self.stems.get(&text::stem(&t)) {
                self.bm25(p, 1.0, &mut s);
            }
        }
        s
    }

    /// Vocabulary terms within trigram-Jaccard `min_sim` of `t`, best first (max `k`).
    pub fn similar_terms(&self, t: &str, min_sim: f64, k: usize) -> Vec<(String, f64)> {
        let q = text::trigrams(t);
        let mut cand: HashMap<usize, usize> = HashMap::new();
        for g in &q {
            for &vi in self.tri.get(g).map(|v| v.as_slice()).unwrap_or(&[]) {
                *cand.entry(vi).or_default() += 1;
            }
        }
        let mut out: Vec<(String, f64)> = cand
            .into_iter()
            .map(|(vi, inter)| {
                let w = &self.vocab[vi];
                let wl = text::trigrams(w).len();
                (w.clone(), inter as f64 / (q.len() + wl - inter) as f64)
            })
            .filter(|(_, s)| *s >= min_sim)
            .collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        out.truncate(k);
        out
    }

    /// Typo-tolerant search: each query term expands to similar vocabulary terms, weighted by similarity.
    pub fn search_fuzzy(&self, terms: &[String], min_sim: f64) -> (HashMap<usize, f32>, Vec<(String, String, f64)>) {
        let mut s = HashMap::new();
        let mut used = Vec::new();
        for t in dedup(terms) {
            for (w, sim) in self.similar_terms(&t, min_sim, 3) {
                if let Some(p) = self.exact.get(&w) {
                    self.bm25(p, sim as f32, &mut s);
                    used.push((t.clone(), w, sim));
                }
            }
        }
        (s, used)
    }

    /// Function nodes ranked by degree: the "most connected memory" used as last-resort context.
    pub fn hubs(g: &Graph, k: usize) -> Vec<usize> {
        let mut f: Vec<usize> = g.functions().collect();
        if f.is_empty() {
            f = (0..g.nodes.len()).collect();
        }
        f.sort_by(|&a, &b| g.degree(b).cmp(&g.degree(a)).then(g.nodes[a].id.cmp(&g.nodes[b].id)));
        f.truncate(k);
        f
    }
}

fn dedup(terms: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    terms.iter().filter(|t| seen.insert(t.as_str())).cloned().collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn mean_len_is_exact_past_f32_integer_range() {
        // 1M nodes of mixed lengths: the total (~21.9M) is past 2^24, where an f32 running sum drifts
        let lens: Vec<f32> = (0..1_000_000u32).map(|i| [3.0f32, 18.0, 22.0, 41.0, 25.5][(i % 5) as usize]).collect();
        let exact = lens.iter().map(|&x| x as f64).sum::<f64>() / lens.len() as f64;
        let naive = lens.iter().sum::<f32>() / lens.len() as f32;
        assert!(((naive as f64) - exact).abs() / exact > 1e-4, "fixture must exercise f32 drift: {naive} vs {exact}");
        assert!(((super::mean_len(&lens) as f64) - exact).abs() / exact < 1e-7);
        assert_eq!(super::mean_len(&[]), 0.0);
    }
}
