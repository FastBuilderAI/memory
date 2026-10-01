//! Passage search (`fastmemory::search`) on a small inline fixture and on fastmemory's own example inputs.
//! Most cases are ported from MahaBodi's tests (github.com/mahabodi/mahabodi, crates/mahabodi-core/tests/memory.rs, MIT).

use fastmemory::ingest::Format;
use fastmemory::search::{Memory, Stage};
use fastmemory::text;

const EXAMPLES: &[&str] = &["business_analytics", "email_analysis", "health_science", "robotics", "world_events"];

fn example(name: &str) -> String {
    std::fs::read_to_string(format!("{}/example/{name}/input.md", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// A small plain-prose fixture: three sources, no markup.
fn fixture() -> Memory {
    Memory::from_documents(&[
        (
            "The U.S. Census Bureau counts the population every ten years. The census also records housing.\n\n\
             Congress uses census counts to apportion seats in the House of Representatives.",
            "census",
        ),
        (
            "Travel must be approved by a manager before booking. Receipts are required for every reimbursement.\n\n\
             Reimbursement requests are paid within five business days.",
            "expense policy",
        ),
        ("Photosynthesis converts light into chemical energy. Chlorophyll absorbs red and blue light.", "biology"),
    ])
}

#[test]
fn plain_prose_becomes_source_scoped_passages() {
    let m = fixture();
    assert_eq!(m.len(), 5, "{:?}", m.atfs().iter().map(|a| &a.id).collect::<Vec<_>>());
    assert!(m.atfs().iter().all(|a| a.action == "Passage"));
    assert_eq!(m.atfs().iter().filter(|a| a.id.starts_with("expense_policy_")).count(), 2);
    // fastmemory's entity-tag parser alone finds nothing in the same prose
    assert!(fastmemory::parser::parse_markdown("Travel must be approved by a manager before booking.").is_empty());
}

#[test]
fn natural_language_questions_find_the_passage() {
    let m = fixture();
    for (q, want) in [
        ("How often does the Census Bureau count the population?", "counts the population"),
        ("Who must approve travel?", "Travel must be approved"),
        ("How long until reimbursement requests are paid?", "paid within five business days"),
        ("What does chlorophyll absorb?", "Chlorophyll absorbs"),
    ] {
        let r = m.search(q, 3);
        assert_eq!(r.stage, Stage::Exact, "{q}");
        assert!(r.matched, "{q}");
        assert!(r.hits[0].text.contains(want), "{q} -> {:?}", r.hits[0]);
        assert_eq!(r.hits[0].score, 1.0);
        assert!(!r.hits[0].block.is_empty());
        assert_eq!(r.hits[0].node, format!("F_{}", r.hits[0].id));
    }
}

#[test]
fn us_style_abbreviations_behave_as_in_mahabodi() {
    let m = fixture();
    // "U.S." splits into 1-char pieces ("u", stopword "s"): no content terms, so nothing to match or cover
    assert!(text::terms("U.S.").is_empty());
    for q in ["U.S.", "u.s", "US", "the U.S."] {
        let r = m.search(q, 3);
        assert_eq!(r.stage, Stage::Hub, "{q}");
        assert!(!r.matched && r.handoff, "{q}");
        assert_eq!(r.term_coverage, 0.0, "{q}");
        assert!(r.hits.iter().all(|h| h.score == 0.0));
    }
    // with content beside it, the abbreviation is simply ignored
    assert_eq!(text::terms("U.S. Census Bureau"), vec!["census", "bureau"]);
    let r = m.search("U.S. Census Bureau", 3);
    assert_eq!(r.stage, Stage::Exact);
    assert!(!r.handoff && r.term_coverage == 1.0);
    assert!(r.hits[0].text.contains("U.S. Census Bureau"));
}

#[test]
fn typos_fall_back_to_fuzzy_with_corrections() {
    let m = fixture();
    // this misspelling already meets the word at the stem stage: both stem to "reimburs"
    assert_eq!(text::stem("reimbursment"), text::stem("reimbursement"));
    let r = m.search("reimbursment", 3);
    assert_eq!(r.stage, Stage::Stem);
    assert!(r.hits[0].text.contains("eimbursement"));
    let r = m.search("chlorophil", 3);
    assert_eq!(r.stage, Stage::Fuzzy);
    assert!(r.matched && !r.handoff);
    assert!(r.corrections.iter().any(|(q, w, s)| q == "chlorophil" && w == "chlorophyll" && *s >= 0.4), "{:?}", r.corrections);
    assert!(r.hits[0].text.contains("Chlorophyll"));
    let r = m.search("photosynthsis", 3);
    assert_eq!(r.stage, Stage::Fuzzy);
    assert!(r.hits[0].text.contains("Photosynthesis"));
}

#[test]
fn stems_and_cjk_are_searchable() {
    let mut m = Memory::new();
    m.ingest("## [ID: Expense_Rules]\n**Action:** Apply_Rules\n**Logic:** Rules for approving travel expenses.\n", Format::Auto, "x");
    m.ingest("报销需要收据。差旅必须事先批准。", Format::Text, "zh");
    let r = m.search("approve", 3);
    assert!(r.matched && matches!(r.stage, Stage::Stem | Stage::Exact), "{:?}", r.stage);
    assert_eq!(r.hits[0].node, "F_Expense_Rules");
    let r = m.search("收据", 3);
    assert_eq!(r.stage, Stage::Exact);
    assert!(r.hits[0].text.contains("收据"));
}

#[test]
fn empty_memory_is_an_answer_not_an_error() {
    let m = Memory::new();
    let r = m.search("anything", 5);
    assert_eq!(r.stage, Stage::Empty);
    assert!(!r.matched && r.handoff && r.hits.is_empty());
    let m = Memory::from_documents(&[("", "x"), ("   \n\n ", "y")]);
    assert!(m.is_empty());
    assert_eq!(m.search("anything", 5).stage, Stage::Empty);
}

#[test]
fn no_match_is_a_reported_hub_fallback() {
    let m = fixture();
    let r = m.search("zzqxv wkkpj", 5);
    assert_eq!(r.stage, Stage::Hub);
    assert!(!r.matched && r.handoff);
    assert!(!r.hits.is_empty() && r.hits.iter().all(|h| h.score == 0.0));
    assert!(r.confidence <= 0.05);
}

fn token_memory() -> Memory {
    let mut m = Memory::new();
    m.ingest("## [ID: Validate_Token]\n**Action:** Validate_Token\n**Data_Connections:** Session_UUID\n**Logic:** Check the customers session token.\n", Format::Auto, "x");
    m
}

#[test]
fn stopword_and_tiny_queries_do_not_confidently_match() {
    let m = token_memory();
    for q in ["to", "e", "in", "the to", "  "] {
        let r = m.search(q, 5);
        assert!(!r.matched && r.handoff, "{q:?} -> {:?} matched={}", r.stage, r.matched);
        assert_eq!(r.term_coverage, 0.0);
    }
}

#[test]
fn one_term_of_many_hands_off() {
    let m = token_memory();
    let r = m.search("refund policy for Khmer wholesale customers", 5);
    assert!(r.matched, "customers is in memory");
    assert!(r.term_coverage < 0.5);
    assert!(r.handoff, "coverage {} must hand off", r.term_coverage);
    let r = m.search("session token", 5);
    assert!(r.matched && !r.handoff);
}

#[test]
fn examples_are_found_by_their_own_terms() {
    for name in EXAMPLES {
        let input = example(name);
        let m = Memory::from_documents(&[(input.as_str(), *name)]);
        assert!(m.len() >= 10, "{name}: {}", m.len());
        // take a real term from the first ATF's logic line and query it
        let first_logic = input.lines().find(|l| l.starts_with("**Logic:**")).unwrap();
        let term = text::terms(first_logic).into_iter().max_by_key(|t| t.len()).unwrap();
        let r = m.search(&term, 5);
        assert!(r.matched, "{name}: {term} -> {:?}", r.stage);
        assert_eq!(r.stage, Stage::Exact);
    }
}

#[test]
fn typo_of_a_real_word_is_corrected() {
    let input = example("health_science");
    let m = Memory::from_documents(&[(input.as_str(), "hs")]);
    // pick a real vocabulary word and misspell it
    let word = text::terms(&input).into_iter().filter(|t| t.len() >= 9 && t.is_ascii()).max().unwrap();
    let typo: String = word.chars().enumerate().filter(|(i, _)| *i != 3).map(|(_, c)| c).collect();
    let r = m.search(&typo, 5);
    assert_eq!(r.stage, Stage::Fuzzy, "{typo}");
    assert!(r.matched);
    assert!(r.corrections.iter().any(|(_, w, _)| *w == word), "{typo} -> {:?}", r.corrections);
}

#[test]
fn hits_are_passages_never_bare_labels() {
    let m = Memory::from_documents(&[(example("robotics").as_str(), "r")]);
    for q in ["mission", "students", "validating spacecraft", "mars"] {
        let r = m.search(q, 5);
        assert!(r.hits.iter().all(|h| h.node.starts_with("F_")), "{q}: {:?}", r.hits.iter().map(|h| &h.node).collect::<Vec<_>>());
    }
}

#[test]
fn isolated_atf_is_found() {
    // ATFs with no Data/Access/Events links: no edges, still searchable in their own block
    let input = "## [ID: Quarterly_Close]\n**Action:** Close_Books\n**Logic:** Reconcile ledgers before the quarterly close deadline.\n\n## [ID: Vendor_Onboarding]\n**Action:** Onboard_Vendor\n**Logic:** Collect tax forms and banking details from each new vendor.\n";
    let m = Memory::from_documents(&[(input, "fin")]);
    assert!(fastmemory::graph::atf_edges(m.atfs()).is_empty());
    let r = m.search("ledgers", 3);
    assert_eq!(r.hits[0].node, "F_Quarterly_Close");
    assert_eq!(r.hits[0].block, "C - Quarterly_Close");
}

#[test]
fn reingesting_replaces_instead_of_duplicating() {
    let mut m = Memory::new();
    m.ingest("## [ID: A]\n**Action:** Old_Thing\n", Format::Auto, "x");
    m.ingest("## [ID: A]\n**Action:** New_Thing\n", Format::Auto, "x");
    assert_eq!(m.len(), 1);
    assert_eq!(m.atfs()[0].action, "New_Thing");
    let mut m = Memory::new();
    m.add_documents(&[("Photosynthesis converts light. Chlorophyll absorbs red light.", "bio")]);
    m.add_documents(&[("Photosynthesis converts light. Chlorophyll absorbs red light.", "bio")]);
    assert_eq!(m.len(), 1);
}

// MahaBodi 0.1.2 regression: two unrelated pages whose prose contains "(Block 4)" collapsed into one ATF "4", the
// second overwriting the first. With entity tags opt-in, both pages keep their own text.
#[test]
fn auto_ingest_keeps_both_documents_with_tag_lookalikes() {
    let m = Memory::from_documents(&[
        ("# Kwun Tong Garden Estate\n\nA public housing estate; Lotus Tower was built in 1987 (Block 4).", "pg134044"),
        ("# Zzz Test Estate\n\nAnother estate, rebuilt in 1990 (Block 4).", "pg999"),
    ]);
    assert!(!m.atfs().iter().any(|a| a.id == "4"), "{:?}", m.atfs());
    let r = m.search("Lotus Tower", 3);
    assert!(r.hits[0].text.contains("Lotus Tower"));
    let r = m.search("rebuilt 1990", 3);
    assert!(r.hits[0].text.contains("rebuilt in 1990"));
}

// Opting in keeps fastmemory's semantics: the same tag name in two documents is one ATF (by design).
#[test]
fn entity_tags_opt_in_merges_names_across_documents() {
    let mut m = Memory::new();
    m.ingest_many(&[
        ("(Component Billing) (Function Charge) uses (Data Card_Token).", Format::EntityTags, "a"),
        ("(Component Billing) (Function Charge) uses (Data Invoice_Total).", Format::EntityTags, "b"),
    ]);
    assert_eq!(m.atfs().iter().filter(|a| a.id == "Charge").count(), 1);
    assert_eq!(m.search("charge", 1).hits[0].node, "F_Charge");
}

#[test]
fn results_are_deterministic_across_builds() {
    // every build uses fresh HashMaps (different random hash seeds): results must not depend on them
    let docs: Vec<(String, String)> = EXAMPLES.iter().map(|n| (example(n), n.to_string())).collect();
    let queries = ["mission", "spacecraft students", "revenue growth", "reimbursment", "zzqxv", "U.S.", "patients treatment"];
    let run = || {
        let m = Memory::from_documents(&docs);
        queries.iter().map(|q| serde_json::to_string(&m.search(q, 10)).unwrap()).collect::<Vec<_>>()
    };
    let first = run();
    for _ in 0..5 {
        assert_eq!(run(), first);
    }
    let m = Memory::from_documents(&docs);
    let blocks = |m: &Memory| m.graph().blocks.iter().map(|b| (b.name.clone(), b.members.len())).collect::<Vec<_>>();
    assert_eq!(blocks(&m), blocks(&Memory::from_documents(&docs)));
}

#[test]
fn ranking_ties_break_by_id() {
    // identical passages under different sources tie on score, level and degree: id order decides
    let m = Memory::from_documents(&[("Orbital tethers lift cargo.", "b"), ("Orbital tethers lift cargo.", "a")]);
    let r = m.search("orbital tethers", 5);
    assert_eq!(r.hits.len(), 2);
    assert_eq!(r.hits[0].score, r.hits[1].score);
    assert!(r.hits[0].id < r.hits[1].id);
}
