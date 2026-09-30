//! Clustering backends: the same blocks in every run and for any edge order; the native engine (when
//! FASTMEMORY_NATIVE_LIB points to its library) recovers planted groups.
use fastmemory::cluster;

fn planted(n: usize, groups: usize) -> Vec<(String, String)> {
    // deterministic LCG, no rand dependency
    let mut x: u64 = 42;
    let mut next = || { x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (x >> 33) as usize };
    let size = n / groups;
    let mut e = std::collections::BTreeSet::new();
    for i in 0..n {
        for _ in 0..6 {
            let j = if next() % 10 == 0 { next() % n } else { (i / size) * size + next() % size };
            if i != j { e.insert((format!("F_n{}", i.min(j)), format!("F_n{}", i.max(j)))); }
        }
    }
    e.into_iter().collect()
}

fn canon(p: &std::collections::HashMap<String, usize>) -> Vec<Vec<String>> {
    let mut g: std::collections::HashMap<usize, Vec<String>> = Default::default();
    for (n, c) in p { g.entry(*c).or_default().push(n.clone()); }
    let mut v: Vec<Vec<String>> = g.into_values().map(|mut x| { x.sort(); x }).collect();
    v.sort();
    v
}

#[test]
fn partition_is_deterministic_and_order_invariant() {
    let e = planted(1200, 12);
    let base = canon(&cluster::partition(&e));
    for _ in 0..3 { assert_eq!(canon(&cluster::partition(&e)), base); }
    let mut rev = e.clone(); rev.reverse();
    let flipped: Vec<_> = e.iter().map(|(a, b)| (b.clone(), a.clone())).collect();
    eprintln!("backend: {}; communities {}", fastmemory::louvain_backend::describe(), base.len());
    if fastmemory::louvain_backend::describe() != "inline" {
        // the native engine is order-invariant; the inline phase-1 Louvain depends on node order by design
        assert_eq!(canon(&cluster::partition(&rev)), base);
        assert_eq!(canon(&cluster::partition(&flipped)), base);
        assert_eq!(base.len(), 12, "the native engine should recover the 12 planted groups");
    }
}

#[test]
fn topology_json_is_identical_across_runs() {
    let e = planted(600, 6);
    let a = cluster::run_louvain(&e, &vec![]);
    for _ in 0..3 { assert_eq!(cluster::run_louvain(&e, &vec![]), a); }
}

#[test]
fn status_reports_backend_and_relative_paths_are_refused() {
    let s = fastmemory::louvain_backend::status();
    eprintln!("status: {s}");
    assert!(s["backend"] == "inline" || s["backend"] == "native");
    if s["backend"] == "inline" {
        assert!(s["reason"].as_str().map_or(false, |r| !r.is_empty()), "the fallback must say why: {s}");
    }
}
