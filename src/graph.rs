// Ported from MahaBodi (github.com/mahabodi/mahabodi, crates/mahabodi-core/src/graph.rs, MIT).
//! The searchable memory graph: nodes, levels, edges and blocks built from `Atf` records.
//!
//! Edges are the ones fastmemory's CLI builds (`F_x -> D_/A_/E_`) plus `F_a -- F_b` context links
//! (`**Context_Links:**` in ATF sections). Blocks are the communities from `cluster::partition` (the
//! native engine when its library is loaded, else the built-in Louvain). Unlike the topology JSON,
//! every ATF is kept as a node even when it has no edges, in a singleton block, so it stays findable.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::parser::Atf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Function,
    Data,
    Access,
    Event,
    Concept,
}

impl Level {
    pub fn from_id(id: &str) -> Level {
        match id.get(..2) {
            Some("F_") => Level::Function,
            Some("D_") => Level::Data,
            Some("A_") => Level::Access,
            Some("E_") => Level::Event,
            _ => Level::Concept,
        }
    }

    /// Levels that carry retrievable content rank first in result lists.
    pub fn rank(self) -> u8 {
        match self {
            Level::Function => 0,
            Level::Data => 1,
            Level::Concept => 2,
            Level::Event => 3,
            Level::Access => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub id: String,
    pub level: Level,
    /// Human label: ATF id + action for functions, bare name otherwise.
    pub label: String,
    /// Retrievable passage/logic text (functions only).
    pub text: String,
    pub block: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Block {
    pub name: String,
    pub members: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub index_of: HashMap<String, usize>,
    pub adj: Vec<Vec<usize>>,
    pub blocks: Vec<Block>,
    pub edge_count: usize,
    /// Lowercased (id, label) per node, computed once at build for the substring rule.
    pub(crate) lower: Vec<(String, String)>,
}

/// fastmemory's edge construction (as in `main.rs` and `lib.rs`): `F_x` to its `D_`, `A_` and `E_` nodes.
pub fn atf_edges(atfs: &[Atf]) -> Vec<(String, String)> {
    let mut edges = Vec::new();
    for atf in atfs {
        let f_id = format!("F_{}", atf.id);
        for link in &atf.data_connections {
            edges.push((f_id.clone(), format!("D_{}", link)));
        }
        for acc in atf.access.split(',') {
            let acc = acc.trim();
            if !acc.is_empty() {
                edges.push((f_id.clone(), format!("A_{}", acc)));
            }
        }
        for ev in atf.events.split(',') {
            let ev = ev.trim();
            if !ev.is_empty() {
                edges.push((f_id.clone(), format!("E_{}", ev)));
            }
        }
    }
    edges
}

/// fastmemory's edges plus `F_a -- F_b` context links whose target is a known ATF.
pub fn all_edges(atfs: &[Atf], links: &[(String, String)]) -> Vec<(String, String)> {
    let mut edges = atf_edges(atfs);
    let known: HashSet<&str> = atfs.iter().map(|a| a.id.as_str()).collect();
    for (a, b) in links {
        if known.contains(b.as_str()) && a != b {
            edges.push((format!("F_{a}"), format!("F_{b}")));
        }
    }
    edges
}

impl Graph {
    pub fn build(atfs: &[Atf], links: &[(String, String)], texts: &HashMap<String, String>) -> Graph {
        let edges = all_edges(atfs, links);
        // communities, grouped and ordered independently of HashMap iteration order
        let part = crate::cluster::partition(&edges);
        let mut grouped: HashMap<usize, Vec<String>> = HashMap::new();
        for (n, c) in part {
            grouped.entry(c).or_default().push(n);
        }
        let mut groups: Vec<Vec<String>> = grouped.into_values().map(|mut v| { v.sort(); v }).collect();
        groups.sort();

        let mut g = Graph::default();
        let actions: HashMap<&str, &Atf> = atfs.iter().map(|a| (a.id.as_str(), a)).collect();

        let add = |g: &mut Graph, id: &str| -> usize {
            if let Some(&i) = g.index_of.get(id) {
                return i;
            }
            let level = Level::from_id(id);
            let bare = id.get(2..).unwrap_or(id);
            let (label, text) = match level {
                Level::Function => {
                    let a = actions.get(bare);
                    let action = a.map(|a| a.action.as_str()).unwrap_or("");
                    let label = if action.is_empty() || action == bare { bare.to_string() } else { format!("{bare} {action}") };
                    let text = texts.get(bare).cloned().unwrap_or_else(|| {
                        a.map(|a| format!("{} {}", a.input, a.logic).trim().to_string()).unwrap_or_default()
                    });
                    (label, text)
                }
                _ => (bare.to_string(), String::new()),
            };
            let i = g.nodes.len();
            g.nodes.push(Node { id: id.to_string(), level, label, text, block: usize::MAX });
            g.index_of.insert(id.to_string(), i);
            g.adj.push(Vec::new());
            i
        };

        // every ATF is a node, connected or not
        for a in atfs {
            add(&mut g, &format!("F_{}", a.id));
        }
        let mut seen: HashSet<(usize, usize)> = HashSet::new();
        for (s, t) in &edges {
            let si = add(&mut g, s);
            let ti = add(&mut g, t);
            let key = (si.min(ti), si.max(ti));
            if si != ti && seen.insert(key) {
                g.adj[si].push(ti);
                g.adj[ti].push(si);
            }
        }
        g.edge_count = seen.len();

        // community membership; block names follow fastmemory's scheme (`cluster::run_louvain`)
        for (ci, ids) in groups.iter().enumerate() {
            let members: Vec<usize> = ids.iter().filter_map(|id| g.index_of.get(id).copied()).collect();
            if members.is_empty() {
                continue;
            }
            let funcs: Vec<&str> = members.iter().filter(|&&m| g.nodes[m].level == Level::Function).map(|&m| &g.nodes[m].id[2..]).collect();
            let name = match funcs.len() {
                1 => format!("C - {}", funcs[0]),
                0 => format!("C - Data_{ci}"),
                _ => format!("C - Community_{ci}"),
            };
            let bi = g.blocks.len();
            for &m in &members {
                g.nodes[m].block = bi;
            }
            g.blocks.push(Block { name, members });
        }
        // isolated nodes (no edges, so absent from the partition) get singleton blocks
        for i in 0..g.nodes.len() {
            if g.nodes[i].block == usize::MAX {
                let bi = g.blocks.len();
                g.nodes[i].block = bi;
                let name = format!("C - {}", g.nodes[i].id.get(2..).unwrap_or(&g.nodes[i].id));
                g.blocks.push(Block { name, members: vec![i] });
            }
        }
        // deterministic block order: by each block's smallest node id
        let mut order: Vec<usize> = (0..g.blocks.len()).collect();
        order.sort_by(|&a, &b| {
            let ka = g.blocks[a].members.iter().map(|&m| g.nodes[m].id.as_str()).min();
            let kb = g.blocks[b].members.iter().map(|&m| g.nodes[m].id.as_str()).min();
            ka.cmp(&kb)
        });
        let mut remap = vec![0; order.len()];
        let mut blocks = Vec::with_capacity(order.len());
        for (new, &old) in order.iter().enumerate() {
            remap[old] = new;
            let mut b = g.blocks[old].clone();
            b.members.sort_unstable();
            blocks.push(b);
        }
        g.blocks = blocks;
        for n in &mut g.nodes {
            n.block = remap[n.block];
        }
        g.lower = g.nodes.iter().map(|n| (n.id.to_lowercase(), n.label.to_lowercase())).collect();
        g
    }

    pub fn degree(&self, i: usize) -> usize {
        self.adj[i].len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn functions(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.nodes.len()).filter(|&i| self.nodes[i].level == Level::Function)
    }
}
