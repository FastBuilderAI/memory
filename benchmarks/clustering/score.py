"""Scoring for PREREG_CLUSTERING.md. Reads work/runs/<arm>/<graph>.json and writes work/results.json.

Per graph and arm: NMI (arithmetic-mean normalised, scikit-learn's default) and ARI against ground truth; modularity Q
on the full graph; stability = mean pairwise NMI between the arm's 5 runs; median wall time and CPU seconds.
LFR: each graph's NMI is averaged over its 5 runs first; then per (n, mu) a paired two-sided Wilcoxon over the 10
graphs, N vs each other arm, Holm-adjusted across the 18 cells within each comparison.
SNAP: primary = disjoint view (each node covered by the top-5000 communities assigned to its largest one; scored on
covered nodes only); secondary = overlapping NMI (McDaid et al., cdlib's MGH variant) against the full top-5000 cover.
"""
import gzip, itertools, json, os, statistics


def _truth_lfr(d):
    with open(os.path.join(d, "community.dat")) as f:
        return {int(a): int(b) for a, b in (l.split()[:2] for l in f)}


def _truth_snap(path):
    with gzip.open(path + ".top5000.cmty.txt.gz", "rt") as f:
        comms = [list(map(int, l.split())) for l in f if l.strip()]
    best = {}
    for c in sorted(comms, key=len):           # larger communities overwrite smaller: each node gets its largest
        for v in c:
            best[v] = id(c)
    return best, comms


def _nmi_ari(truth, part):
    from sklearn.metrics import adjusted_rand_score, normalized_mutual_info_score
    nodes = [v for v in truth if v in part]
    t = [truth[v] for v in nodes]; p = [part[v] for v in nodes]
    return normalized_mutual_info_score(t, p, average_method="arithmetic"), adjusted_rand_score(t, p)


def _modularity(edges, part):
    import networkx as nx
    G = nx.Graph(); G.add_edges_from(edges)
    groups = {}
    for v in G:
        groups.setdefault(part.get(v, ("solo", v)), set()).add(v)
    return nx.community.modularity(G, groups.values())


def _holm(ps):
    order = sorted(range(len(ps)), key=lambda i: ps[i])
    adj, run = [0.0] * len(ps), 0.0
    for rank, i in enumerate(order):
        run = max(run, min(1.0, (len(ps) - rank) * ps[i]))
        adj[i] = run
    return adj


def score(WORK, ARMS, NS, MUS, SEEDS, RUNS):
    from scipy.stats import wilcoxon
    from sklearn.metrics import normalized_mutual_info_score
    import run as harness
    res = {"per_graph": {}, "lfr_cells": {}, "snap": {}, "meta": {}}
    arms = [a for a in ARMS if os.path.isdir(os.path.join(WORK, "runs", a))]
    for a in arms:
        m = os.path.join(WORK, "runs", a, "_meta.json")
        res["meta"][a] = json.load(open(m)) if os.path.exists(m) else None
    for name, path in harness.graphs():
        edges = harness.load_edges(path)
        if name.startswith("lfr/"):
            truth, cover = _truth_lfr(path), None
        else:
            truth, cover = _truth_snap(path)
        row = {}
        for a in arms:
            f = os.path.join(WORK, "runs", a, name.replace("/", "__") + ".json")
            if not os.path.exists(f):
                continue
            doc = json.load(open(f)); runs = doc["runs"]
            parts = [{int(k): v for k, v in r["partition"].items()} for r in runs]
            nmi, ari = zip(*[_nmi_ari(truth, p) for p in parts])
            nodes = sorted(parts[0])
            stab = statistics.mean(normalized_mutual_info_score([p[v] for v in nodes], [q.get(v, -1) for v in nodes])
                                   for p, q in itertools.combinations(parts, 2))
            r = {"nmi_mean": statistics.mean(nmi), "nmi_runs": nmi, "ari_mean": statistics.mean(ari),
                 "q_run0": _modularity(edges, parts[0]), "stability": stab,
                 "wall_s_median": statistics.median(x["wall_s"] for x in runs), "cpu_s_median": statistics.median(x["cpu_s"] for x in runs),
                 "e2e_wall_s_median": statistics.median(x.get("e2e_wall_s", float("nan")) for x in runs),
                 "e2e_cpu_s_median": statistics.median(x.get("e2e_cpu_s", float("nan")) for x in runs),
                 "loadavg_1m_median": statistics.median(x.get("loadavg_1m", float("nan")) for x in runs),
                 "peak_rss_mb": doc.get("peak_rss_mb"), "boot_id": doc.get("boot_id")}
            if cover is not None:
                try:
                    from cdlib import NodeClustering, evaluation
                    import networkx as nx
                    G = nx.Graph(); G.add_edges_from(edges)
                    comms = {}
                    for v, c in parts[0].items():
                        comms.setdefault(c, []).append(v)
                    r["onmi_mgh_run0"] = evaluation.overlapping_normalized_mutual_information_MGH(
                        NodeClustering(list(comms.values()), G), NodeClustering(cover, G)).score
                except ImportError:
                    r["onmi_mgh_run0"] = "cdlib not installed"
            row[a] = r
        res["per_graph"][name] = row
        print("scored", name, flush=True)
    # LFR cells: N vs each other arm, Wilcoxon over the 10 graphs, Holm across the 18 cells per comparison
    for other in [a for a in arms if a != "N"]:
        cells, ps = [], []
        for n in NS:
            for mu in MUS:
                xs, ys = [], []
                for s in SEEDS:
                    g = res["per_graph"].get("lfr/n%d_mu%.1f_s%d" % (n, mu, s), {})
                    if "N" in g and other in g:
                        xs.append(g["N"]["nmi_mean"]); ys.append(g[other]["nmi_mean"])
                if len(xs) < 2:
                    continue
                diff = [x - y for x, y in zip(xs, ys)]
                p = 1.0 if all(d == 0 for d in diff) else wilcoxon(xs, ys, alternative="two-sided").pvalue
                cells.append(("n%d_mu%.1f" % (n, mu), statistics.mean(xs), statistics.mean(ys), len(xs))); ps.append(p)
        for (cell, mx, my, k), p, pa in zip(cells, ps, _holm(ps)):
            verdict = "beat" if (pa < 0.05 and mx > my) else "loss" if (pa < 0.05 and mx < my) else "tie"
            res["lfr_cells"].setdefault(cell, {})["N_vs_" + other] = {"nmi_N": mx, "nmi_other": my, "graphs": k, "p": p,
                                                                     "p_holm": pa, "verdict": verdict}
    # clarification 6.3: the clean single-worker timing subsample (runs_clean/) is reported as timings only,
    # never re-scored; any wall-clock speed ratio at n = 100K quotes these numbers
    clean = {}
    cdir = os.path.join(WORK, "runs_clean")
    if os.path.isdir(cdir):
        for a in sorted(os.listdir(cdir)):
            for f in sorted(os.listdir(os.path.join(cdir, a))):
                if not f.endswith(".json") or f == "_meta.json":
                    continue
                doc = json.load(open(os.path.join(cdir, a, f)))
                clean.setdefault(a, {})[doc["graph"]] = {
                    "wall_s_median": statistics.median(x["wall_s"] for x in doc["runs"]),
                    "e2e_wall_s_median": statistics.median(x.get("e2e_wall_s", float("nan")) for x in doc["runs"]),
                    "cpu_s_median": statistics.median(x["cpu_s"] for x in doc["runs"]),
                    "loadavg_1m_median": statistics.median(x.get("loadavg_1m", float("nan")) for x in doc["runs"])}
    res["clean_timing_subsample"] = clean or None
    man = os.path.join(WORK, "manifest.json")
    res["lfr_checks"] = {k: v.get("check") for k, v in json.load(open(man))["lfr"].items()} if os.path.exists(man) else None
    # the shared box carried another project's load during part of the run: every (graph, arm, run) whose
    # 1-minute load average reached 10, or whose package temperature reached 95 C, is listed so timing readers
    # can see the contention window (quality is unaffected; CPU-seconds are the pre-registered speed metric)
    contended = []
    for a in arms:
        for fn in sorted(os.listdir(os.path.join(WORK, "runs", a))):
            if not fn.endswith(".json") or fn == "_meta.json":
                continue
            doc = json.load(open(os.path.join(WORK, "runs", a, fn)))
            for r in doc.get("runs", []):
                if r.get("loadavg_1m", 0) >= 10 or (r.get("cpu_temp_c") or 0) >= 95:
                    contended.append({"graph": doc["graph"], "arm": a, "run": r["run"],
                                      "loadavg_1m": round(r.get("loadavg_1m", 0), 1), "cpu_temp_c": r.get("cpu_temp_c")})
    res["contention_window"] = {"rule": "loadavg_1m >= 10 or cpu_temp_c >= 95", "n": len(contended), "runs": contended}
    il = os.path.join(WORK, "runs", "_interruptions.jsonl")
    res["interruptions"] = [json.loads(l) for l in open(il)] if os.path.exists(il) else []
    json.dump(res, open(os.path.join(WORK, "results.json"), "w"), indent=1, default=str)
    print("SCORED", os.path.join(WORK, "results.json"))
