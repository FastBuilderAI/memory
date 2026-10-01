# Pre-registration: FastMemory's native clustering engine against Leiden and Louvain

Draft written 2026-10-01, before any graph below has been clustered by any arm, and revised the same day after
review. Changes after it is pushed are made only as dated clarifications, before results are read.

## Question

On standard community-detection benchmarks, how does FastMemory's native clustering engine compare with the current
standard (Leiden) and with common Louvain implementations? The comparison covers:

- quality, against ground-truth communities and by modularity;
- stability across runs and node orderings;
- speed and memory.

## Graphs

1. **LFR benchmark graphs** (Lancichinetti, Fortunato & Radicchi 2008), generated with the original LFR generator (the
   authors' C++ "benchmark" code, compiled on the test machine; its source version and SHA-256 are recorded):
   - standard parameters: ⟨k⟩ = 20, k_max = 50, τ1 = 2, τ2 = 1, and the "small" community range of 10–50 nodes;
   - n ∈ {1,000; 10,000; 100,000};
   - mixing μ ∈ {0.1, 0.2, 0.3, 0.4, 0.5, 0.6}. This grid is fixed now: no μ value is added or dropped after results
     are seen;
   - 10 graphs per (n, μ), generator seeds 1–10. A generator failure at a seed is recorded, and the next seed is
     used.
   - The graphs and their SHA-256 are written before any arm runs.
2. **SNAP graphs with ground-truth communities:** com-Amazon, com-DBLP and com-YouTube. Two views are reported, and
   the primary one is named here:
   - **Primary, disjoint view:** using the top 5,000 ground-truth communities, each covered node is assigned to its
     largest top-5,000 community. Scores are computed **only on the nodes those communities cover** (most nodes are
     in none). The clustering itself always runs on the full graph.
   - **Secondary, overlapping view:** overlapping NMI (McDaid et al.'s ONMI) against the full overlapping
     top-5,000 cover. Every arm produces disjoint communities, so the penalty for that is equal across arms.
   - Modularity Q is always computed on the full graph.
3. **Small reference graphs** (sanity only, no verdict): Zachary's karate club, and Les Misérables.

## Arms (each at its default settings; no tuning on any graph)

- **N (FastMemory native engine):** resolution 1.0, unlimited levels (its defaults).
- **B (FastMemory built-in Louvain):** phase 1 only, as shipped. A **weak baseline**, labelled as such. N vs B is
  never a headline.
- **LD (Leiden, modularity):** `leidenalg`, ModularityVertexPartition, `n_iterations=-1` (until stable), seed = run
  index.
- **LC (Leiden, CPM):** `leidenalg`, CPMVertexPartition, resolution = the graph's edge density, `n_iterations=-1`,
  seed = run index. A second Leiden objective.
- **IM (Infomap):** the `infomap` Python package, default settings, two-level (`--two-level`), seed = run index. It
  is the classic top performer on LFR (Lancichinetti & Fortunato 2009), often strongest at high μ.
- **LV (igraph multilevel):** `community_multilevel`, igraph's C Louvain.
- **NX (networkx Louvain):** `louvain_communities`, resolution 1.0, seed = run index.

Each arm runs single-threaded where the implementation allows it. For every arm, the thread count and the CPU-seconds
are recorded next to the wall time. N is the build without its optional parallel feature, so it runs on one thread. **Every result describes that build.**
Summaries say "N, single-threaded build", and no speed or accuracy from this benchmark is ever quoted for a parallel
build. The parallel build's accuracy is not measured here.
Each graph gets 5 runs per arm, with the node order shuffled by `random.Random(run)`.

## Metrics

- **Quality:** NMI against ground truth, normalised by the arithmetic mean of the entropies (scikit-learn's default
  `normalized_mutual_info_score`); ARI; and modularity Q.
- **Stability:** the mean pairwise NMI between an arm's 5 runs on the same graph (1.0 = identical).
- **Speed:** wall-clock clustering time per run, and the CPU-seconds, excluding graph loading and conversion, measured
  the same way for every arm. Reported as the median of 5 runs. Peak RSS is reported too.

## Verdicts (per graph family and size; nothing pooled across families)

- **Primary: N vs LD, on NMI.**
  - For LFR: per (n, μ), a paired two-sided Wilcoxon signed-rank test over the 10 graphs. Each graph's NMI is first
    averaged over its 5 shuffled runs, then the test runs across graphs.
  - **Holm correction** is applied across the 18 primary LFR cells (3 sizes × 6 μ values). "Beat": N > LD with a
    Holm-adjusted p < 0.05. "Tie": adjusted p ≥ 0.05. "Loss": N < LD with an adjusted p < 0.05. Unadjusted p-values
    are reported too.
  - For SNAP: per graph, the mean NMI with a bootstrap 95 % CI over the 5 runs, reported descriptively.
- **Speed:** the median-time ratio LD / N per graph size, reported as measured. It is not a verdict on its own.
- **Secondary:** N vs IM, N vs LC, N vs LV, N vs NX and N vs B, with the same tests, Holm-corrected within each
  comparison; stability of every arm. N vs IM is reported next to the primary result in every summary.
- **Expectation, stated in advance:** Leiden is the current standard, partly because Louvain can produce badly
  connected communities. A tie or loss against Leiden on quality is plausible, and would be reported as it is.

## Rules

- **Machine:** the Mac mini (M2 Pro, 16 GB), with nothing else running.
- **Provenance:** every result file records:
  - the fastmemory commit;
  - the engine's version string;
  - the versions of networkx, igraph, leidenalg and python;
  - the generator seeds.
- **Engine:** used only as a compiled library, through fastmemory's loader. Its source is not part of this benchmark,
  and nothing here redistributes it.
  - The binary's SHA-256 is recorded in provenance.
  - **Reproducibility:** the binary is not public. Results can be reproduced independently only by reviewers given
    the binary under NDA, and every published summary says so.
  - **Development leakage:** whether the engine was ever tuned or validated on LFR or on these SNAP graphs can't be
    ruled out for a proprietary binary. That is stated with the results.
- **Recomputation:** per-run partitions are saved, and the reviewing agent recomputes every metric before anything is
  published.
- **Losses** are reported as losses, here and on any site or pitch.
