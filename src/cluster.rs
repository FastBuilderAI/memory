use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
struct Edge {
    source: String,
    target: String,
}

/// Pure-Rust inline Louvain community detection — no subprocess spawning.
/// Takes edges and ATFs, returns fully-assembled topology JSON.
pub fn run_louvain_inline(edges: &Vec<(String, String)>, atfs: &Vec<crate::parser::Atf>) -> String {
    if edges.is_empty() {
        return "[]".to_string();
    }

    // ── Step 1: Map string IDs to integer indices ──
    let mut node_to_idx: HashMap<&str, usize> = HashMap::new();
    let mut idx_to_node: Vec<&str> = Vec::new();

    for (src, tgt) in edges {
        for name in [src.as_str(), tgt.as_str()] {
            if !node_to_idx.contains_key(name) {
                let id = idx_to_node.len();
                node_to_idx.insert(name, id);
                idx_to_node.push(name);
            }
        }
    }

    let n = idx_to_node.len();
    if n == 0 {
        return "[]".to_string();
    }

    // ── Step 2: Build adjacency + degree ──
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    let mut total_weight: f64 = 0.0;

    for (src, tgt) in edges {
        let si = node_to_idx[src.as_str()];
        let ti = node_to_idx[tgt.as_str()];
        adj[si].push((ti, 1.0));
        adj[ti].push((si, 1.0));
        total_weight += 1.0;
    }

    let m = total_weight; // total edge weight (each edge counted once)
    if m == 0.0 {
        return "[]".to_string();
    }

    let mut degree: Vec<f64> = vec![0.0; n];
    for i in 0..n {
        degree[i] = adj[i].iter().map(|(_, w)| w).sum();
    }

    // ── Step 3: Louvain Phase 1 — local modularity optimization ──
    let mut community: Vec<usize> = (0..n).collect(); // each node starts in own community

    // Pre-compute community aggregates for speed
    let mut comm_internal: HashMap<usize, f64> = HashMap::new(); // internal edge weight
    let mut comm_total: HashMap<usize, f64> = HashMap::new(); // total degree
    for i in 0..n {
        comm_total.insert(i, degree[i]);
        comm_internal.insert(i, 0.0);
    }

    let mut changed = true;
    let mut iterations = 0;
    while changed && iterations < 20 {
        changed = false;
        iterations += 1;

        for i in 0..n {
            let current_comm = community[i];
            let ki = degree[i];

            // Compute edge weight from node i to each neighboring community
            let mut neighbor_weights: HashMap<usize, f64> = HashMap::new();
            for &(j, w) in &adj[i] {
                *neighbor_weights.entry(community[j]).or_insert(0.0) += w;
            }

            let ki_in_current = *neighbor_weights.get(&current_comm).unwrap_or(&0.0);

            // Remove node i from current community
            let sigma_tot_minus_i = comm_total.get(&current_comm).copied().unwrap_or(0.0) - ki;

            // Compute removal cost
            let remove_cost = ki_in_current / m - ki * sigma_tot_minus_i / (2.0 * m * m);

            let mut best_comm = current_comm;
            let mut best_gain = 0.0;

            for (&target_comm, &ki_in) in &neighbor_weights {
                if target_comm == current_comm {
                    continue;
                }
                let sigma_tot = comm_total.get(&target_comm).copied().unwrap_or(0.0);

                // Gain from adding to target community
                let add_gain = ki_in / m - ki * sigma_tot / (2.0 * m * m);
                let delta_q = add_gain - remove_cost;

                if delta_q > best_gain {
                    best_gain = delta_q;
                    best_comm = target_comm;
                }
            }

            if best_comm != current_comm {
                // Move node i
                *comm_total.entry(current_comm).or_insert(0.0) -= ki;
                *comm_total.entry(best_comm).or_insert(0.0) += ki;
                community[i] = best_comm;
                changed = true;
            }
        }
    }

    // ── Step 4: Build hierarchical output ──
    // Group nodes by community
    let mut communities: HashMap<usize, Vec<String>> = HashMap::new();
    for i in 0..n {
        communities.entry(community[i]).or_default().push(idx_to_node[i].to_string());
    }

    // Build ATF lookup
    let mut atf_map: HashMap<String, serde_json::Value> = HashMap::new();
    for atf in atfs {
        let fid = format!("F_{}", atf.id);
        atf_map.insert(fid.clone(), serde_json::json!({
            "id": fid,
            "topology_level": "Function",
            "action": atf.action,
            "data_connections": atf.data_connections.iter().map(|d| format!("D_{}", d)).collect::<Vec<_>>(),
            "access": atf.access.split(',').map(|s| format!("A_{}", s.trim())).filter(|s| s.len() > 2).collect::<Vec<_>>(),
            "events": atf.events.split(',').map(|s| format!("E_{}", s.trim())).filter(|s| s.len() > 2).collect::<Vec<_>>()
        }));
    }

    // Top-level: components (each community is a component)
    let mut blocks: Vec<serde_json::Value> = Vec::new();

    for (comm_idx, (_comm_id, nodes)) in communities.iter().enumerate() {
        // Separate function nodes from data/access/event nodes
        let mut func_nodes: Vec<String> = Vec::new();
        let mut data_nodes: Vec<String> = Vec::new();

        for node_id in nodes {
            if node_id.starts_with("F_") {
                func_nodes.push(node_id.clone());
            } else {
                data_nodes.push(node_id.clone());
            }
        }

        // Build full node objects
        let full_nodes: Vec<serde_json::Value> = nodes.iter().map(|nid| {
            if let Some(full_obj) = atf_map.get(nid) {
                full_obj.clone()
            } else if nid.starts_with("D_") {
                serde_json::json!({"id": nid, "action": &nid[2..], "topology_level": "Data"})
            } else if nid.starts_with("A_") {
                serde_json::json!({"id": nid, "action": &nid[2..], "topology_level": "Access"})
            } else if nid.starts_with("E_") {
                serde_json::json!({"id": nid, "action": &nid[2..], "topology_level": "Event"})
            } else {
                serde_json::json!({"id": nid, "action": nid, "topology_level": "Unknown"})
            }
        }).collect();

        let block_name = if func_nodes.len() == 1 {
            format!("C - {}", &func_nodes[0][2..]) // strip F_ prefix
        } else if !func_nodes.is_empty() {
            format!("C - Community_{}", comm_idx)
        } else {
            format!("C - Data_{}", comm_idx)
        };

        blocks.push(serde_json::json!({
            "name": block_name,
            "depth": 0,
            "topology_level": "Component",
            "block_type": "topology_memory",
            "nodes": full_nodes,
            "sub_blocks": []
        }));
    }

    serde_json::to_string(&blocks).unwrap_or_else(|_| "[]".to_string())
}
