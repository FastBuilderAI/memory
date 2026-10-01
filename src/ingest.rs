// Ported from MahaBodi (github.com/mahabodi/mahabodi, crates/mahabodi-core/src/ingest.rs, MIT).
//! Turns raw input into `Atf` records.
//!
//! `parser::parse_markdown` only understands entity tags like `(Function Validate_Token)`.
//! The README's Action-Topology Format (`## [ID: x]` + `**Action:**` fields) and plain prose
//! both parse to *zero* ATFs there, which yields an empty graph on which every query fails.
//! `ingest` accepts all three shapes. With `Format::Auto` (the default), ATF sections are
//! detected and everything else becomes source-scoped prose passages; entity tags are parsed
//! only when `Format::EntityTags` is asked for.

use std::collections::HashMap;

use crate::parser::Atf;
use serde::{Deserialize, Serialize};

use crate::text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    /// Detect per input: `## [ID: x]` ATF sections are parsed; everything else is prose. fastmemory
    /// entity tags are NOT auto-detected: prose like "(Block 4)" would otherwise become an ATF keyed by
    /// the bare name, dropping the passage and colliding across documents. Use `EntityTags` for markup.
    #[default]
    Auto,
    /// `## [ID: x]` sections with `**Field:** value` lines.
    Atf,
    /// fastmemory entity tags, parsed by `parser::parse_markdown` (opt-in). Tag names are
    /// global ids by design: the same `(Function X)` in two documents is one ATF.
    EntityTags,
    /// Free prose, chunked into passages.
    Text,
}

#[derive(Debug, Default, Clone)]
pub struct Ingested {
    pub atfs: Vec<Atf>,
    /// Function-to-function links (`**Context_Links:** [A, B]`), as ATF ids.
    pub links: Vec<(String, String)>,
    /// Retrievable text per ATF id.
    pub texts: HashMap<String, String>,
}

impl Ingested {
    pub fn extend(&mut self, other: Ingested) {
        self.atfs.extend(other.atfs);
        self.links.extend(other.links);
        self.texts.extend(other.texts);
    }
}

/// `next_passage` numbers prose passages so ids stay unique across calls.
pub fn ingest(input: &str, format: Format, source: &str, next_passage: &mut usize) -> Ingested {
    let mut out = match format {
        Format::Atf => parse_atf(input),
        Format::EntityTags => parse_entity_tags(input),
        Format::Text => parse_text(input, source, next_passage),
        Format::Auto => {
            // Decide per region, never per document: an ATF section is a `#` heading line
            // carrying `[ID: ...]` up to the next heading; the rest is prose. Entity tags are opt-in
            // (`Format::EntityTags`): in real text "(Block 4)" is prose, not markup.
            let (atf_text, rest) = split_atf_sections(input);
            let mut out = parse_atf(&atf_text);
            let prose: Vec<&str> = rest.split("\n\n").filter(|p| !p.trim().is_empty()).collect();
            out.extend(parse_text(&fold_headings(&prose).join("\n\n"), source, next_passage));
            out
        }
    };
    // Last resort: content must never ingest to nothing.
    if out.atfs.is_empty() && input.chars().any(|c| c.is_alphanumeric()) && format != Format::Text {
        out = parse_text(input, source, next_passage);
    }
    merge_duplicates(out)
}

/// Heading-only paragraphs are not passages: prefix them onto the next prose paragraph
/// ("# Notes" + "Remember the audit." -> "Notes: Remember the audit."), drop trailing ones.
fn fold_headings(paras: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    for p in paras {
        let lines: Vec<&str> = p.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        if !lines.is_empty() && lines.iter().all(|l| l.starts_with('#')) {
            // a later heading starts a new section: it replaces, never stacks on, an earlier one
            pending = lines.iter().map(|l| l.trim_start_matches('#').trim().to_string()).collect();
            continue;
        }
        if pending.is_empty() {
            out.push(p.to_string());
        } else {
            out.push(format!("{}: {}", std::mem::take(&mut pending).join(" / "), p.trim()));
        }
    }
    out
}

/// A complete fastmemory entity tag, i.e. exactly what `parser::parse_markdown` matches.
fn entity_tag_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"\((Component|Block|Function|Data|Access|Event)\s+([A-Za-z0-9_]+)\)").unwrap())
}

fn split_atf_sections(input: &str) -> (String, String) {
    let (mut atf, mut rest) = (String::new(), String::new());
    let mut in_atf = false;
    for line in input.lines() {
        let t = line.trim_start();
        if t.starts_with('#') {
            in_atf = t.contains("[ID:");
        }
        let dst = if in_atf { &mut atf } else { &mut rest };
        dst.push_str(line);
        dst.push('\n');
    }
    (atf, rest)
}

/// Same id ingested twice (Block-anchored paragraphs, repeated ATF ids): one ATF, union of links.
fn merge_duplicates(g: Ingested) -> Ingested {
    let mut order: Vec<String> = Vec::new();
    let mut by_id: HashMap<String, Atf> = HashMap::new();
    for a in g.atfs {
        match by_id.get_mut(&a.id) {
            Some(e) => {
                e.data_connections.extend(a.data_connections);
                e.data_connections.sort();
                e.data_connections.dedup();
                for (dst, src) in [(&mut e.access, a.access), (&mut e.events, a.events)] {
                    let mut items: Vec<String> = dst.split(',').chain(src.split(','))
                        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    items.sort();
                    items.dedup();
                    *dst = items.join(",");
                }
                if e.logic.is_empty() {
                    e.logic = a.logic;
                }
            }
            None => {
                order.push(a.id.clone());
                by_id.insert(a.id.clone(), a);
            }
        }
    }
    let atfs = order.into_iter().map(|id| by_id.remove(&id).unwrap()).collect();
    Ingested { atfs, links: g.links, texts: g.texts }
}

fn parse_entity_tags(input: &str) -> Ingested {
    let atfs = crate::parser::parse_markdown(input);
    // Attach each paragraph's text to the ATF(s) fastmemory anchors it to: its Function
    // tags, else the current Block, else the current Component (fastmemory's own rule).
    let mut texts: HashMap<String, String> = HashMap::new();
    let (mut comp, mut block) = (String::new(), String::new());
    for para in input.split("\n\n") {
        let mut funcs = Vec::new();
        for c in entity_tag_re().captures_iter(para) {
            match &c[1] {
                "Component" => comp = c[2].to_string(),
                "Block" => block = c[2].to_string(),
                "Function" => funcs.push(c[2].to_string()),
                _ => {}
            }
        }
        let targets = if !funcs.is_empty() {
            funcs
        } else if !block.is_empty() {
            vec![block.clone()]
        } else if !comp.is_empty() {
            vec![comp.clone()]
        } else {
            continue;
        };
        let clean = entity_tag_re().replace_all(para, "$2").replace('_', " ");
        for t in targets {
            let e = texts.entry(t).or_default();
            if !e.is_empty() {
                e.push(' ');
            }
            e.push_str(clean.trim());
        }
    }
    Ingested { atfs, links: Vec::new(), texts }
}

fn clean_id(s: &str) -> String {
    s.trim().trim_matches(|c: char| c == '[' || c == ']' || c == '`').trim().to_string()
}

/// Split a field value like `{Expense_Type, Receipt_Image}` or `[A, B]` into items.
fn list_items(v: &str) -> Vec<String> {
    v.split(|c: char| c == ',' || c == ';' || c == '|')
        .map(|s| s.trim().trim_matches(|c: char| "{}[]()`'\"".contains(c)).trim().to_string())
        .filter(|s| !s.is_empty() && s.chars().any(|c| c.is_alphanumeric()))
        .map(|s| s.replace(' ', "_"))
        .collect()
}

fn parse_atf(input: &str) -> Ingested {
    let mut out = Ingested::default();
    let mut cur: Option<(String, Vec<(String, String)>, Vec<String>)> = None;

    let flush = |cur: &mut Option<(String, Vec<(String, String)>, Vec<String>)>, out: &mut Ingested| {
        if let Some((id, fields, free)) = cur.take() {
            let mut atf = Atf {
                id: id.clone(),
                action: String::new(),
                input: String::new(),
                logic: String::new(),
                data_connections: Vec::new(),
                access: String::new(),
                events: String::new(),
            };
            let mut text_parts = Vec::new();
            for (k, v) in &fields {
                text_parts.push(v.clone());
                match k.as_str() {
                    "action" => atf.action = v.clone(),
                    "input" => {
                        atf.input = v.clone();
                        atf.data_connections.extend(list_items(v));
                    }
                    "logic" => atf.logic = v.clone(),
                    "data_connections" | "data" => atf.data_connections.extend(list_items(v)),
                    "access" => atf.access = list_items(v).join(","),
                    "events" | "event" => atf.events = list_items(v).join(","),
                    "context_links" | "links" => {
                        for l in list_items(v) {
                            out.links.push((id.clone(), l));
                        }
                    }
                    _ => {}
                }
            }
            text_parts.extend(free);
            atf.data_connections.sort();
            atf.data_connections.dedup();
            if atf.action.is_empty() {
                atf.action = id.clone();
            }
            out.texts.insert(id.clone(), text_parts.join(" ").trim().to_string());
            out.atfs.push(atf);
        }
    };

    for line in input.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            if let Some(start) = t.find("[ID:") {
                flush(&mut cur, &mut out);
                let rest = &t[start + 4..];
                let id = clean_id(rest.split(']').next().unwrap_or(rest));
                if !id.is_empty() {
                    cur = Some((id, Vec::new(), Vec::new()));
                }
                continue;
            }
        }
        let Some((_, fields, free)) = cur.as_mut() else { continue };
        let body = t.trim_start_matches(['-', '*', ' ']);
        // `**Key:** value`  (after trimming leading `**` the line is `Key:** value`)
        if let Some(pos) = body.find(":**") {
            let key = body[..pos].trim().trim_matches('*').to_lowercase().replace([' ', '-'], "_");
            let val = body[pos + 3..].trim().to_string();
            fields.push((key, val));
        } else if !t.is_empty() {
            free.push(t.to_string());
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// Prose -> passages of up to ~3 sentences / 480 chars, one ATF each. The passage's
/// content terms become its data connections (the role NLTK nouns play in
/// fastmemory's Python extractor), so passages sharing terms cluster together.
fn parse_text(input: &str, source: &str, next_passage: &mut usize) -> Ingested {
    let mut out = Ingested::default();
    let src = {
        let t = text::split_words(source).join("_");
        if t.is_empty() { "doc".to_string() } else { t }
    };
    let mut chunk: Vec<String> = Vec::new();
    let mut chunk_len = 0usize;
    let mut chunks = Vec::new();
    for para in input.split("\n\n") {
        for s in text::sentences(para) {
            if !chunk.is_empty() && (chunk.len() >= 3 || chunk_len + s.len() > 480) {
                chunks.push(chunk.join(" "));
                chunk.clear();
                chunk_len = 0;
            }
            chunk_len += s.len();
            chunk.push(s);
        }
        if !chunk.is_empty() {
            chunks.push(chunk.join(" "));
            chunk.clear();
            chunk_len = 0;
        }
    }
    for c in chunks {
        // content-addressed: re-ingesting the same passage replaces it instead of duplicating it
        let id = format!("{src}_{:08x}", fnv1a(c.as_bytes()) as u32);
        *next_passage += 1;
        let mut data: Vec<String> = text::terms(&c).into_iter().map(|t| capitalize(&t)).collect();
        data.sort();
        data.dedup();
        out.texts.insert(id.clone(), c.clone());
        out.atfs.push(Atf {
            id,
            action: "Passage".into(),
            input: String::new(),
            logic: String::new(),
            data_connections: data,
            access: String::new(),
            events: String::new(),
        });
    }
    out
}

fn fnv1a(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf29ce484222325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x100000001b3))
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ATF: &str = "# Knowledge Base\n\n## [ID: ATF_REF_01]\n**Action:** Provide_Reimbursement_Logic\n**Input:** {Expense_Type, Receipt_Image}\n**Logic:** If Expense_Type == 'Travel', apply Rule_X.\n**Context_Links:** [ATF_REF_02]\n\n## [ID: ATF_REF_02]\n**Action:** Travel Policy\n**Input:** {}\n**Logic:** All travel must be approved.\n**Context_Links:** [ATF_REF_01]\n";

    #[test]
    fn fastmemory_parser_alone_finds_nothing_in_atf_headers() {
        // The gap `ingest` closes: the entity-tag parser alone yields no ATFs for the README's ATF format.
        assert!(crate::parser::parse_markdown(ATF).is_empty());
    }

    #[test]
    fn parses_atf_headers() {
        let mut n = 0;
        let g = ingest(ATF, Format::Auto, "kb", &mut n);
        assert_eq!(g.atfs.len(), 2);
        assert_eq!(g.atfs[0].action, "Provide_Reimbursement_Logic");
        assert_eq!(g.atfs[0].data_connections, vec!["Expense_Type", "Receipt_Image"]);
        assert_eq!(g.links, vec![("ATF_REF_01".into(), "ATF_REF_02".into()), ("ATF_REF_02".into(), "ATF_REF_01".into())]);
        assert!(g.texts["ATF_REF_02"].contains("approved"));
    }

    #[test]
    fn delegates_entity_tags_to_fastmemory() {
        let src = "(Component Auth) (Function Validate_Token) uses (Data Session_UUID) for (Access Role_Admin) on (Event User_Login).";
        let mut n = 0;
        let g = ingest(src, Format::EntityTags, "x", &mut n);
        assert_eq!(g.atfs.len(), 1);
        assert_eq!(g.atfs[0].id, "Validate_Token");
        assert!(g.atfs[0].data_connections.contains(&"Session_UUID".to_string()));
    }

    // Regression from MahaBodi 0.1.2: Wikipedia prose with "(Block 4)" became one ATF with id "4", no passage,
    // and a second page with "(Block 4)" overwrote it. Auto must keep such text as source-scoped passages.
    #[test]
    fn auto_keeps_entity_tag_lookalikes_as_prose() {
        let a = "Kwun Tong Garden Estate is a public housing estate. Lotus Tower was built in 1987 (Block 4).";
        let b = "Another estate was rebuilt in 1990 (Block 4) and 1992 (Block 5).";
        let (mut n1, mut n2) = (0, 0);
        let ga = ingest(a, Format::Auto, "pg1", &mut n1);
        let gb = ingest(b, Format::Auto, "pg2", &mut n2);
        assert!(ga.atfs.iter().all(|x| x.id.starts_with("pg_1_") && x.action == "Passage"), "{:?}", ga.atfs);
        assert!(gb.atfs.iter().all(|x| x.id.starts_with("pg_2_") && x.action == "Passage"), "{:?}", gb.atfs);
        assert!(ga.texts.values().any(|t| t.contains("Lotus Tower")));
        assert!(!ga.atfs.iter().any(|x| x.id == "4" || x.action == "Dynamic Parse"));
    }

    #[test]
    fn chunks_prose() {
        let mut n = 7;
        let g = ingest("Photosynthesis converts light. Chlorophyll absorbs red light.\n\nMitochondria make ATP.", Format::Auto, "bio notes", &mut n);
        assert_eq!(g.atfs.len(), 2);
        assert!(g.atfs[0].id.starts_with("bio_notes_"));
        assert_ne!(g.atfs[0].id, g.atfs[1].id);
        assert_eq!(n, 9);
        // same content, same id
        let mut n2 = 0;
        let again = ingest("Photosynthesis converts light. Chlorophyll absorbs red light.", Format::Text, "bio notes", &mut n2);
        assert_eq!(again.atfs[0].id, g.atfs[0].id);
        assert!(g.atfs[0].data_connections.contains(&"Chlorophyll".to_string()));
    }

    // Regression inputs from review: Auto must never drop prose because of a look-alike marker.
    #[test]
    fn prose_with_partial_entity_tag_is_kept() {
        let mut n = 0;
        let g = ingest("Node runs one thread (Event loop drives all callbacks). Blocking it stalls every request.\n\nUse worker threads for CPU work.", Format::Auto, "node", &mut n);
        assert_eq!(g.atfs.len(), 2);
        assert!(g.texts.values().any(|t| t.contains("worker threads")));
    }

    #[test]
    fn prose_with_inline_id_marker_is_kept() {
        let mut n = 0;
        let g = ingest("Every ticket has a key like [ID: 1234] in its subject line. Refunds take five days.\n\nEscalate after two days.", Format::Auto, "t", &mut n);
        assert_eq!(g.atfs.len(), 2);
        assert!(g.texts.values().any(|t| t.contains("Refunds take five days")));
    }

    #[test]
    fn block_anchored_tags_get_text_and_one_id() {
        let mut n = 0;
        let g = ingest("(Component Billing) (Block Invoices)\n\nInvoices use (Data Invoice_Total) and fire (Event Invoice_Sent).", Format::EntityTags, "b", &mut n);
        let ids: Vec<&str> = g.atfs.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, vec!["Invoices"]);
        assert!(g.atfs[0].data_connections.contains(&"Invoice_Total".to_string()));
        assert!(g.texts["Invoices"].contains("Invoice Total"));
    }

    #[test]
    fn atf_then_heading_prose_becomes_its_own_passage() {
        let mut n = 0;
        let g = ingest(&format!("{ATF}\n# Notes\n\nRemember the quarterly audit."), Format::Auto, "kb", &mut n);
        assert_eq!(g.atfs.len(), 3);
        assert!(!g.texts["ATF_REF_02"].contains("quarterly"));
        assert!(g.texts.values().any(|t| t == "Notes: Remember the quarterly audit."));
    }

    #[test]
    fn empty_input_is_empty_not_error() {
        let mut n = 0;
        assert!(ingest("", Format::Auto, "x", &mut n).atfs.is_empty());
        assert!(ingest("   \n\n  ", Format::Text, "x", &mut n).atfs.is_empty());
    }
}
