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


def _lfr_check(d, n, mu):
    """Clarification 2: the realised graph against the LFR targets (k = 20, maxk = 50, minc = 10, maxc = 50).
    Realised mu = mean over nodes of (edges leaving the node's community / degree), LFR's own definition."""
    com = {}
    for l in open(os.path.join(d, "community.dat")):
        a, b = l.split()[:2]; com[int(a)] = int(b)
    deg, ext = {}, {}
    for l in open(os.path.join(d, "network.dat")):  # each undirected edge is listed from both ends
        a, b = map(int, l.split()[:2])
        deg[a] = deg.get(a, 0) + 1
        ext[a] = ext.get(a, 0) + (com[a] != com[b])
    sizes = {}
    for c in com.values():
        sizes[c] = sizes.get(c, 0) + 1
    return {"target": {"n": n, "mu": mu, "k": 20, "maxk": 50, "minc": 10, "maxc": 50},
            "nodes": len(com), "nodes_with_edges": len(deg), "mu_realised": round(sum(ext[v] / deg[v] for v in deg) / len(deg), 4),
            "mean_degree": round(sum(deg.values()) / len(deg), 3), "max_degree": max(deg.values()), "min_degree": min(deg.values()),
            "communities": len(sizes), "min_community": min(sizes.values()), "max_community": max(sizes.values())}


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
                                                   **json.load(open(os.path.join(d, "gen.json"))),
                                                   "check": _lfr_check(d, n, mu)}
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


def _boot_id():
    try:
        return open("/proc/sys/kernel/random/boot_id").read().strip()
    except OSError:  # macOS: boot time stands in for a boot id
        return subprocess.run(["sysctl", "-n", "kern.boottime"], capture_output=True, text=True).stdout.strip()


def _arm_meta(arm):
    meta = {"arm": arm, "python": sys.version.split()[0], "platform": platform.platform(), "threads_env": {
        k: os.environ.get(k) for k in ("OMP_NUM_THREADS", "RAYON_NUM_THREADS", "OPENBLAS_NUM_THREADS")}}
    if arm in ("N", "B"):
        import fastmemory
        meta["clustering_status"] = json.loads(fastmemory.clustering_status())
        st = meta["clustering_status"]
        assert (arm == "N") == (st["backend"] == "native"), "arm %s ran with backend %s" % (arm, st)
        if st.get("path"):
            meta["engine_sha256"] = sha(st["path"])
            b = st["path"] + ".build.json"  # where the binary came from (source commit, no source)
            meta["engine_build"] = json.load(open(b)) if os.path.exists(b) else None
    return meta


def _arm_file(arm, name):
    return os.path.join(WORK, "runs", arm, name.replace("/", "__") + ".json")


def run_arm(arm, only=None):
    """One arm on every graph (or on graph `only`), 5 shuffled-order runs per graph; one file per graph, written
    atomically once its 5 runs finish, with the boot id, the 1-minute load average per run and the process's peak RSS."""
    out = os.path.join(WORK, "runs", arm); os.makedirs(out, exist_ok=True)
    meta = _arm_meta(arm)
    json.dump(meta, open(os.path.join(out, "_meta.json"), "w"), indent=1)
    for name, path in graphs():
        if only is not None and name != only:
            continue
        f = _arm_file(arm, name)
        if os.path.exists(f):
            continue
        boot = _boot_id()
        edges = load_edges(path)
        rec = []
        for r in range(RUNS):
            w0, c0 = time.perf_counter(), time.process_time()
            part = cluster(arm, edges, r)
            rec.append({"run": r, "wall_s": time.perf_counter() - w0, "cpu_s": time.process_time() - c0,
                        "loadavg_1m": os.getloadavg()[0], "partition": part})
        rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / (1 << 20 if platform.system() == "Darwin" else 1 << 10)
        json.dump({"graph": name, "boot_id": boot, "boot_id_end": _boot_id(), "peak_rss_mb": rss, "runs": rec},
                  open(f + ".tmp", "w"))
        os.replace(f + ".tmp", f)
        print(arm, name, round(sum(x["wall_s"] for x in rec) / RUNS, 3), flush=True)


def run_all(arms):
    """PREREG_CLUSTERING clarification 2: arms interleaved per graph, in an order rotated by the graph's index, each
    (graph, arm) in its own process. A graph counts as done only when every arm's file exists with one boot id; a graph
    found partly done (a crash or reboot mid-graph) is logged and all its arms are re-run in full."""
    done_dir = os.path.join(WORK, "runs", "_done"); os.makedirs(done_dir, exist_ok=True)
    log = os.path.join(WORK, "runs", "_interruptions.jsonl")
    native = os.environ.get("FASTMEMORY_NATIVE_LIB")
    for gi, (name, _) in enumerate(graphs()):
        marker = os.path.join(done_dir, name.replace("/", "__") + ".json")
        if os.path.exists(marker):
            continue
        present = [a for a in arms if os.path.exists(_arm_file(a, name))]
        if present:
            with open(log, "a") as fh:
                fh.write(json.dumps({"graph": name, "found_arms": present, "boot_id_now": _boot_id(),
                                     "boot_ids_found": sorted({json.load(open(_arm_file(a, name)))["boot_id"] for a in present}),
                                     "time": time.strftime("%Y-%m-%d %H:%M:%S"), "action": "re-run all arms"}) + "\n")
            for a in present:
                os.remove(_arm_file(a, name))
        order = arms[gi % len(arms):] + arms[:gi % len(arms)]
        for a in order:
            env = dict(os.environ)
            env.pop("FASTMEMORY_NATIVE_LIB", None); env.pop("FASTMEMORY_CLUSTER", None)
            if a == "N":
                assert native, "arm N needs FASTMEMORY_NATIVE_LIB"
                env["FASTMEMORY_NATIVE_LIB"] = native
            elif a == "B":
                env["FASTMEMORY_CLUSTER"] = "builtin"
            subprocess.check_call([sys.executable, os.path.abspath(__file__), "run", "--arm", a, "--graph", name], env=env)
        boots = {json.load(open(_arm_file(a, name)))["boot_id"] for a in arms} | {json.load(open(_arm_file(a, name)))["boot_id_end"] for a in arms}
        if len(boots) != 1:  # a reboot inside this graph's arms: re-run it on the next pass
            with open(log, "a") as fh:
                fh.write(json.dumps({"graph": name, "boot_ids": sorted(boots), "time": time.strftime("%Y-%m-%d %H:%M:%S"),
                                     "action": "boot id changed within the graph; re-run all arms"}) + "\n")
            for a in arms:
                os.remove(_arm_file(a, name))
            continue
        json.dump({"graph": name, "order": order, "boot_id": boots.pop(), "time": time.strftime("%Y-%m-%d %H:%M:%S")},
                  open(marker, "w"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("phase", choices=["gen", "run", "runall", "score"])
    ap.add_argument("--arm", choices=ARMS)
    ap.add_argument("--graph", default=None, help="run: only this graph (e.g. lfr/n1000_mu0.1_s1)")
    ap.add_argument("--arms", default=",".join(ARMS), help="runall: the arms, comma-separated")
    a = ap.parse_args()
    if a.phase == "gen":
        gen()
    elif a.phase == "run":
        run_arm(a.arm, a.graph)
    elif a.phase == "runall":
        run_all(a.arms.split(","))
    else:
        from score import score
        score(WORK, ARMS, NS, MUS, SEEDS, RUNS)


if __name__ == "__main__":
    main()
