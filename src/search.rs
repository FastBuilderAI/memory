// Ported from MahaBodi (github.com/mahabodi/mahabodi, crates/mahabodi-core/src/query.rs and memory.rs, MIT).
//! Passage search over plain documents: the lexical query cascade.
//!
//! For any query on a non-empty memory the result has at least one hit, and `stage` says how it
//! was found:
//!
//!   exact (BM25, plus a substring boost) -> substring -> stem -> fuzzy (trigram) -> hub
//!
//! `hub` is the explicit low-confidence fallback (most connected passages); it is reported as such,
//! never passed off as a match. An empty memory returns `stage = empty` with no hits.
//!
//! ```
//! use fastmemory::search::{Memory, Stage};
//! let m = Memory::from_documents(&[
//!     ("Travel must be approved by a manager. Receipts are required for every expense.", "policy"),
//!     ("The cafeteria opens at eight.", "notes"),
//! ]);
//! let r = m.search("who approves travel?", 3);
//! assert_eq!(r.stage, Stage::Exact);
//! assert!(r.hits[0].text.contains("Travel must be approved"));
//! ```

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::graph::{Graph, Level};
use crate::index::Index;
use crate::ingest::{self, Format};
use crate::parser::Atf;
use crate::text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Exact,
    Substring,
    Stem,
    Fuzzy,
    Hub,
    /// The memory holds nothing.
    Empty,
}

impl Stage {
    fn base_confidence(self) -> f64 {
        match self {
            Stage::Exact => 1.0,
            Stage::Substring => 0.8,
            Stage::Stem => 0.75,
            Stage::Fuzzy => 0.5,
            Stage::Hub => 0.05,
            Stage::Empty => 0.0,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Exact => "exact",
            Stage::Substring => "substring",
            Stage::Stem => "stem",
            Stage::Fuzzy => "fuzzy",
            Stage::Hub => "hub",
            Stage::Empty => "empty",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    /// Passage id: the ATF id (for prose, `<source>_<content hash>`).
    pub id: String,
    /// Graph node id (`F_<id>`).
    pub node: String,
    pub label: String,
    pub text: String,
    /// Relative to the top hit (1.0); 0.0 for every hub hit.
    pub score: f64,
    /// Name of the block (community) holding the passage.
    pub block: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub query: String,
    pub stage: Stage,
    /// False for `hub` and `empty`: nothing in memory matched the query.
    pub matched: bool,
    /// True when the result should not be acted on as an answer: no match, or a match covering
    /// under half of the query's content terms.
    pub handoff: bool,
    /// Fraction of the query's content terms that matched something (0.0 when it has none).
    pub term_coverage: f64,
    /// Stage confidence scaled by coverage: base * (0.5 + 0.5 * coverage).
    pub confidence: f64,
    pub hits: Vec<SearchHit>,
    /// Query term -> vocabulary term substitutions made by the fuzzy stage, with their similarity.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub corrections: Vec<(String, String, f64)>,
}

/// Run the lexical cascade on a built graph and index.
pub fn search(g: &Graph, ix: &Index, q: &str, k: usize) -> SearchResult {
    let k = k.max(1);
    let mut res = SearchResult {
        query: q.to_string(),
        stage: Stage::Empty,
        matched: false,
        handoff: true,
        term_coverage: 0.0,
        confidence: 0.0,
        hits: Vec::new(),
        corrections: Vec::new(),
    };
    if g.is_empty() {
        return res;
    }
    let terms = text::terms(q);
    // A query with no content terms (stopwords, 1-char pieces such as "U.S.") has nothing to cover: 0.0, not 1.0.
    let covered = |scores_terms: &dyn Fn(&str) -> bool| -> f64 {
        if terms.is_empty() {
            0.0
        } else {
            terms.iter().filter(|t| scores_terms(t)).count() as f64 / terms.len() as f64
        }
    };

    let mut scores = ix.search_exact(&terms);
    // the substring rule only for queries that carry content: "to" or "e" would otherwise
    // "match" every id containing those letters
    let substring = if terms.is_empty() || q.trim().chars().count() < 3 { Vec::new() } else { substring_hits(g, q) };
    let (stage, coverage) = if !scores.is_empty() {
        boost(&mut scores, &substring);
        (Stage::Exact, covered(&|t| ix.has_term(t)))
    } else if !substring.is_empty() {
        scores = substring.iter().map(|&i| (i, 1.0f32)).collect();
        (Stage::Substring, covered(&|t| substring.iter().any(|&i| {
            let n = &g.nodes[i];
            n.id.to_lowercase().contains(t) || n.label.to_lowercase().contains(t)
        })))
    } else {
        scores = ix.search_stem(&terms);
        if !scores.is_empty() {
            (Stage::Stem, covered(&|t| !ix.search_stem(&[t.to_string()]).is_empty()))
        } else {
            let (s, used) = ix.search_fuzzy(&terms, 0.4);
            scores = s;
            if !scores.is_empty() {
                let hit_terms: HashSet<&str> = used.iter().map(|(t, _, _)| t.as_str()).collect();
                let cov = covered(&|t| hit_terms.contains(t));
                res.corrections = used;
                (Stage::Fuzzy, cov)
            } else {
                scores = Index::hubs(g, k).into_iter().map(|i| (i, 0.0f32)).collect();
                (Stage::Hub, 0.0)
            }
        }
    };

    // Hits are passages (Function nodes): a matched data/access/event node passes its score to
    // the passages it links (split by its degree), so "Receipt" finds the passages that mention
    // receipts rather than a bare label.
    if stage != Stage::Hub {
        let mut spread: HashMap<usize, f32> = HashMap::new();
        // accumulate in node order: f32 sums in a different order can round differently and swap near-tied hits
        let mut entries: Vec<(usize, f32)> = scores.iter().map(|(&i, &s)| (i, s)).collect();
        entries.sort_unstable_by_key(|e| e.0);
        for (i, s) in entries {
            if g.nodes[i].level == Level::Function {
                *spread.entry(i).or_default() += s;
            } else {
                let fs: Vec<usize> = g.adj[i].iter().copied().filter(|&j| g.nodes[j].level == Level::Function).collect();
                let share = 0.5 * s / (fs.len() as f32).sqrt().max(1.0);
                for f in fs {
                    *spread.entry(f).or_default() += share;
                }
            }
        }
        if !spread.is_empty() {
            scores = spread;
        }
    }
    // deterministic ranking: score, then level, then degree, then id
    let mut ranked: Vec<(usize, f32)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(g.nodes[a.0].level.rank().cmp(&g.nodes[b.0].level.rank()))
            .then(g.degree(b.0).cmp(&g.degree(a.0)))
            .then(g.nodes[a.0].id.cmp(&g.nodes[b.0].id))
    });
    ranked.truncate(k);

    let top = ranked.first().map(|r| r.1).unwrap_or(0.0).max(1e-9);
    for (i, s) in &ranked {
        let n = &g.nodes[*i];
        res.hits.push(SearchHit {
            id: n.id.get(2..).unwrap_or(&n.id).to_string(),
            node: n.id.clone(),
            label: n.label.clone(),
            text: n.text.clone(),
            score: if stage == Stage::Hub { 0.0 } else { (*s / top) as f64 },
            block: g.blocks[n.block].name.clone(),
        });
    }
    res.stage = stage;
    res.matched = !matches!(stage, Stage::Hub | Stage::Empty);
    // matching one term of a many-term query is not an answer to it, whatever the stage
    res.handoff = !res.matched || coverage < 0.5;
    res.term_coverage = coverage;
    res.confidence = (stage.base_confidence() * (0.5 + 0.5 * coverage)).min(1.0);
    res
}

/// fastmemory's matching rule: case-insensitive substring of the whole query in a node's id or label.
fn substring_hits(g: &Graph, q: &str) -> Vec<usize> {
    let ql = q.trim().to_lowercase();
    if ql.is_empty() {
        return Vec::new();
    }
    g.lower
        .iter()
        .enumerate()
        .filter(|(_, (id, label))| id.contains(&ql) || label.contains(&ql))
        .map(|(i, _)| i)
        .collect()
}

/// Exact BM25 hits that also match the whole query as a substring get 0.25 x the top score.
fn boost(scores: &mut HashMap<usize, f32>, substring: &[usize]) {
    let top = scores.values().cloned().fold(0.0f32, f32::max);
    for &i in substring {
        *scores.entry(i).or_default() += 0.25 * top;
    }
}

/// Searchable memory over plain documents: ingested ATFs, and the graph and index built from them.
#[derive(Debug, Default, Clone)]
pub struct Memory {
    atfs: Vec<Atf>,
    links: Vec<(String, String)>,
    texts: HashMap<String, String>,
    graph: Graph,
    index: Index,
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from `(text, source)` documents, each ingested with `Format::Auto`.
    pub fn from_documents<T: AsRef<str>, S: AsRef<str>>(docs: &[(T, S)]) -> Self {
        let mut m = Memory::new();
        m.add_documents(docs);
        m
    }

    /// Add `(text, source)` documents with `Format::Auto`, then rebuild once.
    pub fn add_documents<T: AsRef<str>, S: AsRef<str>>(&mut self, docs: &[(T, S)]) {
        let docs: Vec<(&str, Format, &str)> = docs.iter().map(|(t, s)| (t.as_ref(), Format::Auto, s.as_ref())).collect();
        self.ingest_many(&docs);
    }

    /// Ingest one document in the given format, then rebuild.
    pub fn ingest(&mut self, text: &str, format: Format, source: &str) {
        self.ingest_many(&[(text, format, source)]);
    }

    /// Ingest several documents with ONE rebuild. An ATF id that is defined again (the same prose
    /// passage from the same source, or a repeated ATF id) replaces the earlier one and its links.
    pub fn ingest_many(&mut self, docs: &[(&str, Format, &str)]) {
        for (input, format, source) in docs {
            let mut n = 0usize;
            let got = ingest::ingest(input, *format, source, &mut n);
            let new_ids: HashSet<&str> = got.atfs.iter().map(|a| a.id.as_str()).collect();
            self.atfs.retain(|a| !new_ids.contains(a.id.as_str()));
            self.links.retain(|(a, _)| !new_ids.contains(a.as_str()));
            self.atfs.extend(got.atfs);
            self.links.extend(got.links);
            self.texts.extend(got.texts);
        }
        self.rebuild();
    }

    fn rebuild(&mut self) {
        self.graph = Graph::build(&self.atfs, &self.links, &self.texts);
        self.index = Index::build(&self.graph);
    }

    /// Top `k` passages for `query` (at least one hit on a non-empty memory; see `SearchResult::stage`).
    pub fn search(&self, query: &str, k: usize) -> SearchResult {
        search(&self.graph, &self.index, query, k)
    }

    /// Number of passages (ATFs).
    pub fn len(&self) -> usize {
        self.atfs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.atfs.is_empty()
    }

    pub fn atfs(&self) -> &[Atf] {
        &self.atfs
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Retrievable text of a passage by its id.
    pub fn text_of(&self, id: &str) -> Option<&str> {
        self.texts.get(id).map(String::as_str)
    }
}
