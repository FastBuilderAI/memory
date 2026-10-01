"""PREREG_CLUSTERING.md harness: FastMemory's native engine vs Leiden, Infomap and Louvain.

  gen      build the original LFR generator (authors' C++ code via skojaku/LFR-benchmark @ LFR_COMMIT) and write the
           LFR graphs with pre-registered seeds; download the SNAP graphs and their top-5000 communities
  run      one arm on every graph (5 shuffled-order runs each), in its own process: partitions, wall time,
           CPU seconds, peak RSS, thread count
  score    NMI (arithmetic), ARI, Q, ONMI (SNAP), stability; Wilcoxon per LFR cell with Holm across the 18 cells

    python benchmarks/clustering/run.py gen
    FASTMEMORY_NATIVE_LIB=/abs/libfastmemory_native.dylib python benchmarks/clustering/run.py run --arm N
    FASTMEMORY_CLUSTER=builtin python benchmarks/clustering/run.py run --arm B
    python benchmarks/clustering/run.py run --arm LD   (LC, IM, LV, NX)
    python benchmarks/clustering/run.py score
"""
import argparse, gzip, hashlib, json, os, platform, random, resource, subprocess, sys, time, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
WORK = os.environ.get("CLUSTER_WORK", os.path.join(HERE, "work"))
LFR_REPO, LFR_COMMIT = "https://github.com/skojaku/LFR-benchmark", "b5a7a6a19df1"
NS, MUS, SEEDS, RUNS = (1000, 10000, 100000), (0.1, 0.2, 0.3, 0.4, 0.5, 0.6), range(1, 11), 5
SNAP = {"amazon": "com-amazon", "dblp": "com-dblp", "youtube": "com-youtube"}
ARMS = ("N", "B", "LD", "LC", "IM", "LV", "NX")


def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


# PREREG_CLUSTERING clarification 1: four int functions in the authors' code end without a return. Current GCC turns
# that into a trap (SIGILL at -O0) or an endless loop (-O3), so `return 0;` is added at exactly these closing braces.
# The return values are never used; the patched code writes byte-identical graphs to the unpatched code (checked).
MISSING_RETURNS = (("print.cpp", 12), ("print.cpp", 21), ("histograms.cpp", 643), ("histograms.cpp", 672))


def _patch_missing_returns(src):
    out = {}
    for f, line in MISSING_RETURNS:
        p = os.path.join(src, "src", f)
        before = sha(p)
        lines = open(p).read().split("\n")
        if lines[line - 1] == "}":
            lines[line - 1] = "return 0; }"
            open(p, "w").write("\n".join(lines))
        elif lines[line - 1] != "return 0; }":
            raise RuntimeError("unexpected line %s:%d: %r" % (f, line, lines[line - 1]))
        out["%s:%d" % (f, line)] = {"before": before, "after": sha(p)}
    return out


def gen():
    os.makedirs(WORK, exist_ok=True)
    src = os.path.join(WORK, "lfr-src")
    if not os.path.isdir(src):
        subprocess.check_call(["git", "clone", "-q", LFR_REPO, src])
        subprocess.check_call(["git", "-C", src, "checkout", "-q", LFR_COMMIT])
    patch = _patch_missing_returns(src)
    binp = os.path.join(src, "benchmark")
    if not os.path.exists(binp):
        subprocess.check_call(["g++", "-O3", "-o", binp, os.path.join(src, "src", "benchm.cpp")])
    man = {"lfr_source": {"repo": LFR_REPO, "commit": LFR_COMMIT, "benchm.cpp": sha(os.path.join(src, "src", "benchm.cpp")),
                          "binary": sha(binp), "compiler": subprocess.run(["g++", "--version"], capture_output=True,
                                                                         text=True).stdout.splitlines()[0],
                          "missing_return_patch": patch}, "lfr": {}, "snap": {}}
    for n in NS:
        for mu in MUS:
            for s in SEEDS:
                d = os.path.join(WORK, "lfr", "n%d_mu%.1f_s%d" % (n, mu, s))
                if not os.path.exists(os.path.join(d, "network.dat")):
                    os.makedirs(d, exist_ok=True)
                    seed, tries = s, 0
                    while True:  # a generator failure at a seed is recorded and the next seed is used (prereg)
                        open(os.path.join(d, "time_seed.dat"), "w").write(str(seed))
                        r = subprocess.run([binp, "-N", str(n), "-k", "20", "-maxk", "50", "-t1", "2", "-t2", "1", "-mu", str(mu),
                                            "-minc", "10", "-maxc", "50"], cwd=d, capture_output=True)
                        if r.returncode == 0 and os.path.exists(os.path.join(d, "network.dat")):
                            break
                        tries += 1; seed += 1000
                        if tries > 20:
                            raise RuntimeError("LFR generation failed: %s" % d)
                    json.dump({"seed_used": seed, "failures": tries}, open(os.path.join(d, "gen.json"), "w"))
                man["lfr"][os.path.basename(d)] = {"network": sha(os.path.join(d, "network.dat")),
                                                   "community": sha(os.path.join(d, "community.dat")),
                                                   **json.load(open(os.path.join(d, "gen.json")))}
                print("lfr", os.path.basename(d), flush=True)
    for k, name in SNAP.items():
        d = os.path.join(WORK, "snap"); os.makedirs(d, exist_ok=True)
        for suf in (".ungraph.txt.gz", ".top5000.cmty.txt.gz"):
            p = os.path.join(d, name + suf)
            if not os.path.exists(p):
                urllib.request.urlretrieve("https://snap.stanford.edu/data/bigdata/communities/" + name + suf, p)
            man["snap"][name + suf] = sha(p)
        print("snap", name, flush=True)
    json.dump(man, open(os.path.join(WORK, "manifest.json"), "w"), indent=1)


def graphs():
    """(name, edges as (int, int), ground truth) for every graph."""
    for n in NS:
        for mu in MUS:
            for s in SEEDS:
                d = os.path.join(WORK, "lfr", "n%d_mu%.1f_s%d" % (n, mu, s))
                yield "lfr/" + os.path.basename(d), d
    for name in SNAP.values():
        yield "snap/" + name, os.path.join(WORK, "snap", name)


def load_edges(path):
    if path.endswith("snap/" + os.path.basename(path)) and os.path.exists(path + ".ungraph.txt.gz"):
        with gzip.open(path + ".ungraph.txt.gz", "rt") as f:
            return [tuple(map(int, l.split())) for l in f if l[0] != "#"]
    with open(os.path.join(path, "network.dat")) as f:
        e = [tuple(map(int, l.split()[:2])) for l in f]
    return [(a, b) for a, b in e if a < b]  # LFR lists each undirected edge twice


def cluster(arm, edges, run):
    """-> {node: community}; node order shuffled by random.Random(run) before clustering."""
    nodes = sorted({x for e in edges for x in e})
    perm = nodes[:]; random.Random(run).shuffle(perm)
    pos = {v: i for i, v in enumerate(perm)}           # relabel = shuffled order
    E = [(pos[a], pos[b]) for a, b in edges]
    random.Random(run).shuffle(E)
    if arm in ("N", "B"):
        import fastmemory
        part = json.loads(fastmemory.cluster_partition([(str(a), str(b)) for a, b in E]))
        lab = {int(k): v for k, v in part.items()}
    elif arm in ("LD", "LC", "LV", "IM"):
        import igraph as ig
        g = ig.Graph(n=len(nodes), edges=E)
        if arm == "LD":
            import leidenalg as la
            mem = la.find_partition(g, la.ModularityVertexPartition, n_iterations=-1, seed=run).membership
        elif arm == "LC":
            import leidenalg as la
            dens = 2 * g.ecount() / (g.vcount() * (g.vcount() - 1))
            mem = la.find_partition(g, la.CPMVertexPartition, resolution_parameter=dens, n_iterations=-1, seed=run).membership
        elif arm == "LV":
            random.seed(run)
            mem = g.community_multilevel().membership
        else:
            import infomap
            im = infomap.Infomap("--two-level --silent --seed %d" % run)
            for a, b in E:
                im.add_link(a, b)
            im.run()
            m = im.get_modules()
            mem = [m.get(i, -1 - i) for i in range(len(nodes))]
        lab = dict(enumerate(mem))
    elif arm == "NX":
        import networkx as nx
        G = nx.Graph(); G.add_nodes_from(range(len(nodes))); G.add_edges_from(E)
        lab = {v: c for c, comm in enumerate(nx.community.louvain_communities(G, resolution=1.0, seed=run)) for v in comm}
    inv = {i: v for v, i in pos.items()}
    return {inv[i]: c for i, c in lab.items()}


def run_arm(arm):
    out = os.path.join(WORK, "runs", arm); os.makedirs(out, exist_ok=True)
    meta = {"arm": arm, "python": sys.version.split()[0], "platform": platform.platform(), "threads_env": {
        k: os.environ.get(k) for k in ("OMP_NUM_THREADS", "RAYON_NUM_THREADS", "OPENBLAS_NUM_THREADS")}}
    if arm in ("N", "B"):
        import fastmemory
        meta["clustering_status"] = json.loads(fastmemory.clustering_status())
        st = meta["clustering_status"]
        assert (arm == "N") == (st["backend"] == "native"), "arm %s ran with backend %s" % (arm, st)
        if st.get("path"):
            meta["engine_sha256"] = sha(st["path"])
    for name, path in graphs():
        f = os.path.join(out, name.replace("/", "__") + ".json")
        if os.path.exists(f):
            continue
        edges = load_edges(path)
        rec = []
        for r in range(RUNS):
            w0, c0 = time.perf_counter(), time.process_time()
            part = cluster(arm, edges, r)
            rec.append({"run": r, "wall_s": time.perf_counter() - w0, "cpu_s": time.process_time() - c0,
                        "loadavg_1m": os.getloadavg()[0], "partition": part})
        json.dump({"graph": name, "runs": rec}, open(f, "w"))
        print(arm, name, round(sum(x["wall_s"] for x in rec) / RUNS, 3), flush=True)
    meta["peak_rss_mb"] = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / (1 << 20 if platform.system() == "Darwin" else 1 << 10)
    json.dump(meta, open(os.path.join(out, "_meta.json"), "w"), indent=1)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("phase", choices=["gen", "run", "score"])
    ap.add_argument("--arm", choices=ARMS)
    a = ap.parse_args()
    if a.phase == "gen":
        gen()
    elif a.phase == "run":
        run_arm(a.arm)
    else:
        from score import score
        score(WORK, ARMS, NS, MUS, SEEDS, RUNS)


if __name__ == "__main__":
    main()
