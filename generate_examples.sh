#!/bin/bash
set -e

cd /Users/pkpro/upperspace3/fastmemory
mkdir -p example/health_science example/robotics example/driverless_cars example/audit example/email_analysis example/business_analytics example/world_events

function write_html {
    local folder=$1
    cat << 'EOF' > "$folder/index.html"
<!DOCTYPE html>
<html>
<head>
    <title>FastMemory Visualization</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <style>
        body { font-family: -apple-system, sans-serif; margin: 0; padding: 20px; background: #0f172a; color: white; }
        .node { stroke: #fff; stroke-width: 1.5px; }
        .link { stroke: #334155; stroke-opacity: 0.6; }
        svg { background: #1e293b; border-radius: 8px; width: 100%; height: 800px; }
        .label { font-size: 12px; fill: white; pointer-events: none; }
    </style>
</head>
<body>
    <h2>Clustered Text Memory Graph (Louvain)</h2>
    <div id="chart"></div>
    <script>
        fetch('output.json').then(r => r.json()).then(data => {
            const nodes = [];
            const links = [];
            
            // Flatten blocks into nodes
            data.forEach((block, i) => {
                nodes.push({ id: block.id, group: i, name: block.name, size: block.node_count * 10 + 10 });
                block.nodes.forEach(n => {
                    nodes.push({ id: n, group: i, name: n, size: 5 });
                    links.push({ source: n, target: block.id, value: 1 });
                });
            });

            const width = document.getElementById('chart').clientWidth;
            const height = 800;

            const simulation = d3.forceSimulation(nodes)
                .force("link", d3.forceLink(links).id(d => d.id).distance(50))
                .force("charge", d3.forceManyBody().strength(-200))
                .force("center", d3.forceCenter(width / 2, height / 2));

            const svg = d3.select("#chart").append("svg")
                .attr("viewBox", [0, 0, width, height]);

            const link = svg.append("g")
                .attr("class", "link")
                .selectAll("line")
                .data(links)
                .join("line");

            const node = svg.append("g")
                .attr("class", "node")
                .selectAll("circle")
                .data(nodes)
                .join("circle")
                .attr("r", d => d.size)
                .attr("fill", d => d3.schemeCategory10[d.group % 10])
                .call(drag(simulation));

            node.append("title").text(d => d.name);

            const label = svg.append("g")
                .attr("class", "label")
                .selectAll("text")
                .data(nodes.filter(n => n.size > 5)) // Only label blocks
                .join("text")
                .text(d => d.name)
                .attr("x", 8)
                .attr("y", 3);

            simulation.on("tick", () => {
                link
                    .attr("x1", d => d.source.x)
                    .attr("y1", d => d.source.y)
                    .attr("x2", d => d.target.x)
                    .attr("y2", d => d.target.y);
                node
                    .attr("cx", d => d.x)
                    .attr("cy", d => d.y);
                label
                    .attr("x", d => d.x + 12)
                    .attr("y", d => d.y + 4);
            });

            function drag(simulation) {
                function dragstarted(event) {
                    if (!event.active) simulation.alphaTarget(0.3).restart();
                    event.subject.fx = event.subject.x;
                    event.subject.fy = event.subject.y;
                }
                function dragged(event) {
                    event.subject.fx = event.x;
                    event.subject.fy = event.y;
                }
                function dragended(event) {
                    if (!event.active) simulation.alphaTarget(0);
                    event.subject.fx = null;
                    event.subject.fy = null;
                }
                return d3.drag()
                    .on("start", dragstarted)
                    .on("drag", dragged)
                    .on("end", dragended);
            }
        });
    </script>
</body>
</html>
EOF
}

# HEALTH SCIENCE (e.g. PubMed QA graph vs Vector)
cat << 'EOF' > example/health_science/input.md
# Health Science: Precision Medicine & Oncology
Original Vector Approach: Embedded clinical trials independently. RAG failed on multi-drug interactions.
FastMemory Approach: Converts Drug mechanisms and patient profiles into ATFs with contextual graph links.

## [ID: ATF_DRUG_001]
**Action:** Administrate_Paclitaxel
**Input:** {Patient_Age, Tumor_Stage}
**Logic:** Inhibit microtubule breakdown.
**Context_Links:** [ATF_SIDE_001], [ATF_MOA_001]

## [ID: ATF_SIDE_001]
**Action:** Monitor_Neuropathy
**Input:** {Patient_Symptoms}
**Logic:** If tingling, reduce dose.
**Context_Links:** [ATF_DRUG_001]

## [ID: ATF_MOA_001]
**Action:** Inhibit_Mitosis
**Input:** {Cellular_Process}
**Logic:** Stops tumor cell division.
**Context_Links:** [ATF_DRUG_001]
EOF

# ROBOTICS
cat << 'EOF' > example/robotics/input.md
# Robotics: Swarm Navigation
Original Vector Approach: Nav-logs embedded. Failed to map conditional failure states (e.g. "If sensor dead, follow neighbour").
FastMemory Approach: Conditionals are parsed as functional ATFs with explicit dependency edges.

## [ID: ATF_NAV_001]
**Action:** Sensor_Fusion_Logic
**Input:** {LidarData, CameraData}
**Logic:** Merge into local costmap.
**Context_Links:** [ATF_FAIL_001]

## [ID: ATF_FAIL_001]
**Action:** Fallback_To_Swarm
**Input:** {Sensor_Status}
**Logic:** If Lidar dead, tracking neighbor bot ID.
**Context_Links:** [ATF_NAV_001]
EOF

# DRIVERLESS CARS
cat << 'EOF' > example/driverless_cars/input.md
# Driverless Cars: Perception & Scenario Trees
Original Vector Approach: Scene graphs clustered via point clouds. 
FastMemory Approach: Semantic rules of the road converted to interconnected ATFs.

## [ID: ATF_SCENE_PEDESTRIAN]
**Action:** Yield_To_Pedestrian
**Input:** {Crosswalk_Zone, Velocity}
**Logic:** Must stop if pedestrian in zone.
**Context_Links:** [ATF_ACT_BRAKE]

## [ID: ATF_ACT_BRAKE]
**Action:** Apply_Braking_Force
**Input:** {Deceleration_Target}
**Logic:** Ramp up brake pressure smoothly.
**Context_Links:** [ATF_SCENE_PEDESTRIAN]
EOF

# AUDIT
cat << 'EOF' > example/audit/input.md
# Audit: Financial Fraud Detection (FinQA)
Previous: Flattened ledger parsing with RAG finding textual matches.
FastMemory: ATFs tracing the flow of funds and compliance rules.

## [ID: ATF_COMPLY_SOX]
**Action:** Verify_SOX_Compliance
**Input:** {Report_Q3}
**Logic:** Check signatures from CEO/CFO.
**Context_Links:** [ATF_LEDGER_REVIEW]

## [ID: ATF_LEDGER_REVIEW]
**Action:** Audit_Expense_Anomalies
**Input:** {T_E_Ledger}
**Logic:** Flag if expense > $10k without receipt.
**Context_Links:** [ATF_COMPLY_SOX]
EOF

# EMAIL ANALYSIS
cat << 'EOF' > example/email_analysis/input.md
# Email: Enron Corporate Communications
Previous Approach: NLP classification (BERT).
FastMemory: Email threads modeled as conversational logic blocks.

## [ID: ATF_THREAD_102]
**Action:** Discuss_Offshore_Accounts
**Input:** {Financial_Strategy}
**Logic:** Move assets to subsidiary.
**Context_Links:** [ATF_THREAD_103]

## [ID: ATF_THREAD_103]
**Action:** Authorize_Transfer
**Input:** {Approval_Chain}
**Logic:** CFO signature required.
**Context_Links:** [ATF_THREAD_102]
EOF

# BUSINESS ANALYTICS
cat << 'EOF' > example/business_analytics/input.md
# Business Analytics: Churn Prediction
Previous: Decision trees on tabular data.
FastMemory: Customer behavior rules synthesized into Actionable Text Functions.

## [ID: ATF_CHURN_DETECT]
**Action:** Detect_Usage_Drop
**Input:** {Weekly_Logins}
**Logic:** If login drop > 50%, flag risk.
**Context_Links:** [ATF_RETENTION_OFFER]

## [ID: ATF_RETENTION_OFFER]
**Action:** Trigger_Discount_Email
**Input:** {Customer_ID}
**Logic:** Send 20% off coupon.
**Context_Links:** [ATF_CHURN_DETECT]
EOF

# WORLD EVENTS
cat << 'EOF' > example/world_events/input.md
# World Events: Geopolitical Supply Chain Mapping
Previous Approach: Knowledge Graphs (Neo4j).
FastMemory: Event nodes clustered dynamically by Louvain communities tracking cascading economic effects.

## [ID: ATF_EVENT_PORT_STRIKE]
**Action:** Delay_Logistics
**Input:** {West_Coast_Ports}
**Logic:** Add 14 days to shipping ETAs.
**Context_Links:** [ATF_IMPACT_AUTO]

## [ID: ATF_IMPACT_AUTO]
**Action:** Adjust_Manufacturing_Forecast
**Input:** {Component_Inventory}
**Logic:** Reduce output by 10%.
**Context_Links:** [ATF_EVENT_PORT_STRIKE]
EOF


for domain in health_science robotics driverless_cars audit email_analysis business_analytics world_events; do
    echo "Processing $domain..."
    cargo run -q -- example/$domain/input.md > example/$domain/output.json
    write_html example/$domain
    
    cat << EOF > example/$domain/README.md
# $domain - FastMemory Example

## Original Huggingface Scenario
In many Huggingface test datasets involving **$domain**, the original approach relies on standard Vector embeddings (RAG) or static Knowledge Graphs. These approaches often fail with multi-hop reasoning or conditional logic because they retrieve flat snippets based on semantic similarity rather than traversing logical dependencies.

## FastMemory "Clustered Text Memory" Approach
Here we convert the dataset rules into **Atomic Text Functions (ATFs)** and use the strict \`rust-louvain\` graph clustering to map out executable semantic structures.

## Reproducible Command
\`\`\`bash
# From fastmemory root:
cargo run -- example/$domain/input.md > example/$domain/output.json
\`\`\`

## Provable Output
\`\`\`json
$(cat example/$domain/output.json | python3 -m json.tool)
\`\`\`

## Visualization
Open \`index.html\` in a browser to view the Force-Directed graph of the clustered logic components generated by the underlying \`rust-louvain\` engine.
EOF

done
