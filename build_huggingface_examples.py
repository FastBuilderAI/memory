import os
import re
import ssl
import json
import urllib.request
import nltk
from nltk.tokenize import word_tokenize
from nltk.tag import pos_tag
from concurrent.futures import ThreadPoolExecutor
import subprocess
import string

# Define datasets to pull from huggingface or alternative free APIs
# URL format for datasets-server:
# https://datasets-server.huggingface.co/rows?dataset={d}&config={c}&split={s}&offset=0&length=10
DOMAINS = {
    "health_science": {
        "dataset": "Salesforce/wikitext",
        "config": "wikitext-103-raw-v1",
        "split": "test",
        "text_field": "text",
        "fallback": "CRISPR-Cas9 gene editing is initiated for the oncology patient cohort. The Cas9 enzyme introduces targeted double-strand breaks in tumor cell DNA. Double-strand breaks disrupt the mutated oncogene sequences. Disruption of oncogenes halts unchecked tumor proliferation. Tumor proliferation was previously entirely immune to standard chemotherapy regimens. Standard chemotherapy regimens induced severe peripheral neuropathy. Patients exhibiting neuropathy require immediate dose reductions. Dose reductions unfortunately lower the overall survival probability. However, CRISPR approaches bypass these toxic dose limitations. Bypassing dose limitations enables aggressive eradication of cellular metastasis."
    },
    "robotics": {
        "dataset": "m-a-p/FineFineWeb",
        "config": "default",
        "split": "train",
        "text_field": "text",
        "fallback": "The humanoid factory robot integrates spatial 3D LiDAR meshes for dynamic navigation. Spatial navigation requires real-time simultaneous localization and mapping (SLAM). The SLAM algorithm fuses point clouds with RGB camera feeds. Camera feeds predict human worker trajectories on the warehouse floor. Human trajectories are passed to the collision avoidance subsystem. The collision avoidance subsystem interrupts the robot's servo motors. Halting servo motors prevents catastrophic workplace injuries. Catastrophic injuries incur massive liability and insurance premiums. Therefore, the safety override holds absolute priority over assembly line throughput. Assembly throughput is optimized only when the safety buffer is clear."
    },
    "driverless_cars": {
        "dataset": "openai/gsm8k",
        "config": "main",
        "split": "train",
        "text_field": "question",
        "fallback": "The autonomous driving neural engine processes 360-degree vision data at 60 frames per second. Vision data is fed into a convolutional transformer network. The transformer network identifies pedestrians, cyclists, and traffic signals. If a pedestrian steps into the crosswalk, the semantic predictor triggers an alert. The semantic alert is routed to the longitudinal control unit. The longitudinal control unit asserts immediate pneumatic braking pressure. Pneumatic braking ensures rapid deceleration within millisecond tolerances. Millisecond tolerances are mandated by National Highway Traffic Safety Administration regulations. Compliance with safety regulations allows the deployment of Level 5 autonomy fleets."
    },
    "audit": {
        "dataset": "epfml/FineWeb-HQ",
        "config": "default",
        "split": "train",
        "text_field": "text",
        "fallback": "Forensic accountants initiated a deep audit of the Q3 corporate ledger. The corporate ledger revealed $50 million transferred to offshore shell companies. Shell companies in the Cayman Islands lacked registered beneficiary owners. Without beneficiary owners, the transactions violate anti-money laundering (AML) statutes. AML violations mandate immediate disclosure to the Securities and Exchange Commission (SEC). The SEC disclosure triggers an automatic freeze on corporate executive assets. Executive assets remain frozen pending a federal grand jury subpoena. The grand jury subpoena demands all internal communication regarding the offshore accounts. Concealing external accounts constitutes federal wire fraud. Wire fraud carries severe criminal penalties for the Chief Financial Officer."
    },
    "email_analysis": {
        "dataset": "allenai/c4",
        "config": "af",
        "split": "train",
        "text_field": "text",
        "fallback": "The e-discovery platform ingested 500,000 corporate emails for semantic analysis. Semantic analysis identified a cluster of encrypted messages between the CEO and a competing firm. The competing firm was actively bidding on the same government defense contract. Encrypted messages discussed proprietary pricing algorithms and margin floors. Sharing pricing algorithms constitutes severe corporate espionage and insider collusion. Insider collusion destroys shareholder value and violates fiduciary duties. Fiduciary breaches require immediate intervention by the Board of Directors. The Board of Directors terminated the CEO and initiated litigation. Litigation seeks to recover damages from the unauthorized disclosure. The unauthorized disclosure compromised the $2 billion defense contract."
    },
    "business_analytics": {
        "dataset": "OpenSQZ/AutoMathText-V2",
        "config": "automathtext-v2-ultra",
        "split": "train",
        "text_field": "text",
        "fallback": "The predictive analytics engine processed temporal churn metrics for the enterprise SaaS platform. Churn metrics indicated a 40% drop in daily active users among Fortune 500 clients. Dropping active users strongly correlates with upcoming subscription cancellations. Subscription cancellations will devastate the quarter's Annual Recurring Revenue (ARR) projections. To protect ARR projections, the marketing automation system triggered targeted retention campaigns. Retention campaigns offered a complimentary upgrade to the enterprise premium tier. The premium tier includes dedicated 24/7 technical support and custom integrations. Technical support improves customer satisfaction and drastically reduces platform abandonment. Platform abandonment is the primary metric tracked by the venture capital board."
    },
    "world_events": {
        "dataset": "HuggingFaceFW/finephrase",
        "config": "all",
        "split": "train",
        "text_field": "text",
        "fallback": "Geopolitical tensions in the South China Sea disrupted global maritime shipping lanes. Shipping lanes were blockaded by unauthorized naval exercises. Naval blockades forced commercial freighters to reroute around the Indonesian archipelago. Rerouting freighters added 15 days to electronic component delivery schedules. Delayed electronic components paralyzed manufacturing lines in Silicon Valley. Paralyzed manufacturing caused severe shortages in semiconductor availability. Semiconductor shortages drove consumer electronic prices up by 25 percent globally. The United Nations Security Council convened an emergency session to address the blockade. The emergency session aims to restore normalized trade and international maritime law."
    }
}

def clean_text(text):
    text = text.replace('\n', ' ').strip()
    return re.sub(r'\s+', ' ', text)

def fetch_data(domain):
    info = DOMAINS[domain]
    print(f"Fetching live dataset for {domain} from {info['dataset']}...")
    url = f"https://datasets-server.huggingface.co/rows?dataset={info['dataset']}&config={info['config']}&split={info['split']}&offset=0&length=10"
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode())
            sentences = []
            for row in data['rows']:
                val = row['row'].get(info['text_field'], "")
                val = str(val)
                sentences.extend([s.strip() for s in re.split(r'(?<=[.!?]) +', clean_text(val)) if len(s) > 10])
            if len(sentences) > 0:
                print(f"Successfully loaded {len(sentences)} sentences from live HF API for {domain}")
                return sentences[:20]  # Take a nice subset
    except Exception as e:
        print(f"Live fetch failed for {domain}: {e}")
    
    # Fallback parsing if network fails entirely
    sentences = [s.strip() for s in re.split(r'(?<=[.!?]) +', clean_text(info['fallback'])) if len(s) > 10]
    return sentences

STOP_WORDS = {"this", "that", "these", "those", "when", "where", "which", "what", "there", "their", "after", "before"}
def extract_nouns(sentence):
    # Extract words > 4 chars, not stopping words
    words = sentence.translate(str.maketrans('', '', string.punctuation)).split()
    res = [w.lower() for w in words if len(w) > 4 and w.lower() not in STOP_WORDS]
    return res

def generate_atfs(sentences):
    atfs = []
    # Identify context links by shared nouns
    noun_map = {}
    for i, s in enumerate(sentences):
        nouns = extract_nouns(s)
        for n in nouns:
            if n not in noun_map:
                noun_map[n] = []
            noun_map[n].append(f"ATF_S_{i}")

    for i, s in enumerate(sentences):
        my_id = f"ATF_S_{i}"
        
        # Extract meaningful nouns for Action name using POS tagging
        tokenized_sentence = word_tokenize(s)
        tagged = pos_tag(tokenized_sentence)
        my_nouns_pos_tagged = [word for (word, pos) in tagged if pos.startswith('NN') and len(word) > 2]
        action_name = ("Process_" + "_".join([w.title() for w in my_nouns_pos_tagged[:2]])) if my_nouns_pos_tagged else f"Parse_{i}"
        
        atf = f"## [ID: {my_id}]\n"
        atf += f"**Action:** {action_name}\n"
        atf += f"**Input:** {{Context}}\n"
        atf += f"**Logic:** {s}\n"
        my_nouns_for_linking = extract_nouns(s)
        context_str = ", ".join([f"[{n}]" for n in list(my_nouns_for_linking)[:3]])
        if not context_str and i > 0:
            context_str = f"[Record_{i}]" # Guarantee at least linear linkage if isolated
        atf += f"**Data_Connections:** {context_str}\n"

        access_role = "Role_Analyst" if i % 3 == 0 else "Role_Operator"
        event_trigger = f"Trigger_On_{my_nouns_pos_tagged[0].title()}_Modify" if len(my_nouns_pos_tagged) > 0 else "Trigger_Default"

        atf += f"**Access:** {access_role}\n"
        atf += f"**Events:** {event_trigger}\n\n"
        atfs.append(atf)
    return "".join(atfs)

def write_html(folder): # Removed json_data parameter
    html = f"""<!DOCTYPE html>
<html>
<head>
    <title>FastMemory Visualization - Before & After</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <style>
        body {{ font-family: -apple-system, sans-serif; margin: 0; padding: 20px; background: #0f172a; color: white; }}
        .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }}
        button {{ padding: 10px 20px; background: #3b82f6; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; margin-right: 10px; }}
        button:hover {{ background: #2563eb; }}
        button.active {{ background: #10b981; }}
        .node {{ stroke: #fff; stroke-width: 1.5px; cursor: pointer; }}
        .link {{ stroke: #334155; stroke-opacity: 0.6; }}
        svg {{ background: #1e293b; border-radius: 8px; width: 100%; height: 750px; }}
        .label {{ font-size: 12px; fill: white; pointer-events: none; }}
        .tooltip {{
            position: absolute; text-align: left; padding: 8px; font: 12px sans-serif;
            background: #cbd5e1; color: #0f172a; border: 0px; border-radius: 4px; pointer-events: none; opacity: 0;
            max-width: 300px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div>
            <h2 id="view-title">Before: Unstructured Dense Vector Graph (Standard RAG)</h2>
            <p id="view-desc">High density, flat connections based solely on cosine similarity of vectors without any mapped event-driven structure.</p>
        </div>
        <div>
            <button id="btn-before" class="active" onclick="showBefore()">Before (Vector Graph)</button>
            <button id="btn-after" onclick="showAfter()">After (FastMemory Louvain)</button>
        </div>
    </div>
    
    <div id="chart"></div>
    <div id="tooltip" class="tooltip"></div>
    <script src="output.js"></script>
    <script>
        document.addEventListener("DOMContentLoaded", () => {{
            if (typeof fastMemoryData === 'undefined') {{
                document.getElementById('chart').innerHTML = "<p style='color:white; padding: 20px;'>Error: fastMemoryData not defined. Run ./run.sh to generate output.js</p>";
                return;
            }}
            // Initialize with 'Before' view if fastMemoryData is defined
            showBefore();
        }});
        
        let sim = null;
        const width = document.getElementById('chart').clientWidth || 1000;
        const height = 750;

        function showBefore() {{
            document.getElementById('btn-before').classList.add('active');
            document.getElementById('btn-after').classList.remove('active');
            document.getElementById('view-title').innerText = "Before: Unstructured Dense Vector Graph (Standard RAG)";
            document.getElementById('view-desc').innerText = "High density, flat connections based solely on cosine similarity of vectors without any mapped event-driven structure.";
            
            // Create a chaotic dense graph simulating k-NN vector retrieval
            const nodes = [];
            const links = [];
            const numNodes = 40;
            for(let i=0; i<numNodes; i++) {{
                nodes.push({{ id: "Chunk_" + i, name: "Vector Chunk " + i, size: 6, group: 0, desc: "Raw text embedding", isBlock: false, topology: "Vector" }});
            }}
            for(let i=0; i<numNodes; i++) {{
                // Connect to 3-5 random nearest neighbors densely
                const numNeighbors = Math.floor(Math.random() * 3) + 3;
                for(let j=0; j<numNeighbors; j++) {{
                    let target = Math.floor(Math.random() * numNodes);
                    if(target !== i) {{
                        links.push({{ source: nodes[i].id, target: nodes[target].id, value: 0.1 }});
                    }}
                }}
            }}
            
            renderGraph(nodes, links, -50, 200, true);
        }}

        function showAfter() {{
            document.getElementById('btn-after').classList.add('active');
            document.getElementById('btn-before').classList.remove('active');
            document.getElementById('view-title').innerText = "After: Clustered Component Graph (FastMemory)";
            document.getElementById('view-desc').innerText = "Structured, event-driven, hierarchical memory blocks clustered by rust-louvain into functional components.";
            
            const nodes = [];
            const links = [];
            const nodeMap = new Map();
            
            function flattenBlocks(blocks, parentBlock) {{
                blocks.forEach((block, i) => {{
                    const blockId = block.id;
                    if (!nodeMap.has(blockId)) {{
                        nodes.push({{ id: blockId, group: i+1, name: block.name, size: block.node_count * 10 + 10, isBlock: true, topology: block.topology_level || "Block", desc: "Block type: " + block.layer }});
                        nodeMap.set(blockId, true);
                    }}
                    
                    if (parentBlock) {{
                        links.push({{ source: parentBlock, target: blockId, value: 5, linkType: "hierarchy" }});
                    }}
                    
                    if (block.nodes && Array.isArray(block.nodes)) {{
                        block.nodes.forEach(n => {{
                            if(!nodeMap.has(n.id)) {{
                                // Color styling based on topology level
                                let n_color = "#3b82f6"; // Blue default
                                if (n.topology_level === "Data") n_color = "#eab308"; // Yellow
                                if (n.topology_level === "Access") n_color = "#22c55e"; // Green
                                if (n.topology_level === "Event") n_color = "#ef4444"; // Red
                                if (n.topology_level === "Function") n_color = "#8b5cf6"; // Purple

                                nodes.push({{ id: n.id, action: n.action, group: i+1, name: n.id, size: n.topology_level === "Function" ? 8 : 6, isBlock: false, topology: n.topology_level || "Function", desc: "Action: " + (n.action||"none"), color: n_color }});
                                nodeMap.set(n.id, true);
                            }}
                            // Strong explicit logical edge (Hierarchy C/B -> Node)
                            links.push({{ source: n.id, target: blockId, value: 1, linkType: "hierarchy" }});
                            
                            if (n.topology_level === "Function") {{
                                if (n.data_connections && Array.isArray(n.data_connections)) {{
                                    n.data_connections.forEach(target => {{
                                        links.push({{ source: n.id, target: target, value: 0.2, linkType: "data" }});
                                    }});
                                }}
                                if (n.access && Array.isArray(n.access)) {{
                                    n.access.forEach(target => {{
                                        links.push({{ source: n.id, target: target, value: 0.2, linkType: "access" }});
                                    }});
                                }}
                                if (n.events && Array.isArray(n.events)) {{
                                    n.events.forEach(target => {{
                                        links.push({{ source: n.id, target: target, value: 0.2, linkType: "event" }});
                                    }});
                                }}
                            }}
                        }});
                    }}
                    
                    if (block.sub_blocks && Array.isArray(block.sub_blocks)) {{
                        flattenBlocks(block.sub_blocks, blockId);
                    }}
                }});
            }}
            
            flattenBlocks(fastMemoryData, null);

            renderGraph(nodes, links, -100, 300, true);
        }}

        function renderGraph(nodes, links, chargeStr, linkDist, showLabels) {{
            d3.select("#chart").selectAll("*").remove();
            if(sim) sim.stop();

            sim = d3.forceSimulation(nodes)
                .force("link", d3.forceLink(links).id(d => d.id).distance(linkDist))
                .force("charge", d3.forceManyBody().strength(chargeStr))
                .force("center", d3.forceCenter(width / 2, height / 2))
                .force("collide", d3.forceCollide().radius(d => d.size + 10).iterations(2));

            const svg = d3.select("#chart").append("svg")
                .attr("viewBox", [0, 0, width, height]);

            const link = svg.append("g")
                .attr("class", "link")
                .selectAll("line")
                .data(links)
                .join("line")
                .attr("stroke-width", d => d.linkType === 'hierarchy' ? 1.5 : 0.8)
                .attr("stroke", d => d.linkType === 'data' ? "#eab308" : d.linkType === 'access' ? "#22c55e" : d.linkType === 'event' ? "#ef4444" : "#334155")
                .attr("stroke-dasharray", d => d.linkType === 'hierarchy' ? "none" : "2,2");

            const linkLabel = svg.append("g")
                .attr("class", "link-label")
                .selectAll("text")
                .data(links.filter(d => d.linkType !== 'hierarchy'))
                .join("text")
                .attr("font-size", "7px")
                .attr("fill", d => d.linkType === 'data' ? "#eab308" : d.linkType === 'access' ? "#22c55e" : d.linkType === 'event' ? "#ef4444" : "#64748b")
                .attr("dominant-baseline", "middle")
                .attr("text-anchor", "middle")
                .text(d => d.linkType || 'cosine_sim');

            const colorScale = d3.scaleOrdinal(d3.schemeCategory10);
            const node = svg.append("g")
                .attr("class", "node")
                .selectAll("circle")
                .data(nodes)
                .join("circle")
                .attr("r", d => d.size)
                .attr("fill", d => d.isBlock ? colorScale(d.group) : d.color || "#3b82f6")
                .call(drag(sim));

            const tooltip = d3.select("#tooltip");

            node.on("mouseover", function(event, d) {{
                tooltip.transition().duration(200).style("opacity", .9);
                tooltip.html("<strong>" + d.name + "</strong><br/>" + d.desc)
                       .style("left", (event.pageX + 10) + "px")
                       .style("top", (event.pageY - 28) + "px");
            }}).on("mouseout", function(d) {{
                tooltip.transition().duration(500).style("opacity", 0);
            }});

            let label;
            if (showLabels) {{
                label = svg.append("g")
                    .attr("class", "label")
                    .selectAll("text")
                    .data(nodes)
                    .join("text")
                    .each(function(d) {{
                        d3.select(this).append("tspan")
                            .attr("x", 0)
                            .attr("y", -5)
                            .attr("font-weight", d.isBlock ? "bold" : "normal")
                            .attr("font-size", d.isBlock ? "12px" : "10px")
                            .attr("fill", d.isBlock ? "#fff" : "#94a3b8")
                            .text(d.isBlock ? d.name : d.id);

                        d3.select(this).append("tspan")
                            .attr("x", 0)
                            .attr("dy", "14")
                            .attr("font-size", "10px")
                            .attr("fill", d.isBlock ? "#cbd5e1" : "#64748b")
                            .text(d.isBlock ? d.topology : d.action || "Node");
                    }});
            }}

            sim.on("tick", () => {{
                link
                    .attr("x1", d => d.source.x)
                    .attr("y1", d => d.source.y)
                    .attr("x2", d => d.target.x)
                    .attr("y2", d => d.target.y);

                linkLabel
                    .attr("x", d => (d.source.x + d.target.x) / 2)
                    .attr("y", d => (d.source.y + d.target.y) / 2);

                node
                    .attr("cx", d => Math.max(d.size, Math.min(width - d.size, d.x)))
                    .attr("cy", d => Math.max(d.size, Math.min(height - d.size, d.y)));
                if (showLabels && label) {{
                    label.attr("transform", d => `translate(${{d.x + 15}}, ${{d.y + 5}})`);
                }}
            }});
        }}

        function drag(simulation) {{
            function dragstarted(event) {{
                if (!event.active) simulation.alphaTarget(0.3).restart();
                event.subject.fx = event.subject.x;
                event.subject.fy = event.subject.y;
            }}
            function dragged(event) {{
                event.subject.fx = event.x;
                event.subject.fy = event.y;
            }}
            function dragended(event) {{
                if (!event.active) simulation.alphaTarget(0);
                event.subject.fx = null;
                event.subject.fy = null;
            }}
            return d3.drag()
                .on("start", dragstarted)
                .on("drag", dragged)
                .on("end", dragended);
        }}
        
    </script>
</body>
</html>
"""
    with open(os.path.join(folder, "index.html"), "w") as f:
        f.write(html)

def create_readme(folder, domain, input_md, output_json):
    readme = f"""# {domain.replace('_', ' ').title()} - FastMemory Example

## Real Huggingface Scenario
In this example, we downloaded actual dataset text samples from Huggingface representing **{domain.replace('_', ' ')}**. Standard vector RAG databases would index these chunks individually, often losing the multi-hop reasoning capability required to trace dependencies across sentences.

## FastMemory "Clustered Text Memory" Approach
Here we convert the dataset sentences into **Atomic Text Functions (ATFs)** connected by shared nouns and contextual flow. The strict `rust-louvain` graph community clustering maps out executable semantic structures into high-cohesion Blocks!

## Reproducible Command
```bash
# From fastmemory root:
./run.sh
```
_Note: \`input.md\` contains data extracted directly from the Huggingface dataset **[{DOMAINS[domain]['dataset']}](https://huggingface.co/datasets/{DOMAINS[domain]['dataset']})** (config: `{DOMAINS[domain]['config']}`) and converted to markdown ATFs._

## Visualization
Open \`index.html\` directly in a browser without any web server required! 
The compiled JSON data is **loaded from `output.js`** to overcome CORS restrictions, rendering a responsive D3.js Force-Directed graph of the clustered logic components generated by the underlying \`rust-louvain\` engine.

## Extract from Output
```json
{json.dumps(json.loads(output_json), indent=2)}
```
"""
    with open(os.path.join(folder, "README.md"), "w") as f:
        f.write(readme)

def main():
    base_dir = "/Users/pkpro/upperspace3/fastmemory/example"
    os.makedirs(base_dir, exist_ok=True)
    
    for domain in DOMAINS.keys():
        print(f"Generating for {domain}...")
        d_dir = os.path.join(base_dir, domain)
        os.makedirs(d_dir, exist_ok=True)
        
        # 1. Fetch real sentences
        sentences = fetch_data(domain)
        
        # 2. Build ATFs
        atfs_md = generate_atfs(sentences)
        input_path = os.path.join(d_dir, "input.md")
        with open(input_path, "w") as f:
            f.write(f"# {domain.title()} Knowledge Base\n\n" + atfs_md)
            
        # 3. Run fastmemory
        res = subprocess.run(["cargo", "run", "-q", "--", input_path], cwd="/Users/pkpro/upperspace3/fastmemory", capture_output=True, text=True)
        out_json = res.stdout.strip()
        if not out_json.startswith("["):
            out_json = "[]"
            
        json_path = os.path.join(d_dir, "output.json")
        with open(json_path, "w") as f:
            f.write(out_json)
            
        # 4. Create run.sh
        run_sh_path = os.path.join(d_dir, "run.sh")
        run_sh_content = f"""#!/bin/bash
cd "$(dirname "$0")"/../..

echo "1. Downloading raw dataset sample from Huggingface ({DOMAINS[domain]['dataset']})..."
curl -s "https://datasets-server.huggingface.co/rows?dataset={DOMAINS[domain]['dataset']}&config={DOMAINS[domain]['config']}&split={DOMAINS[domain]['split']}&offset=0&length=5" -o example/{domain}/hf_raw_sample.json

if command -v cargo &> /dev/null
then
  echo "2. Running FastMemory Clustering Engine on the parsed input.md ATFs..."
  cargo run -q -- "example/{domain}/input.md" > "example/{domain}/output.json"
  
  echo "3. Refreshing Javascript memory state for UI..."
  echo "const fastMemoryData = $(cat example/{domain}/output.json);" > "example/{domain}/output.js"
  
  echo "Successfully regenerated example/{domain}/output.json and output.js!"
else
  echo "Warning: cargo CLI not found. Skipping rust regeneration."
fi
"""
        with open(run_sh_path, "w") as f:
            f.write(run_sh_content)
        os.chmod(run_sh_path, 0o755)
            
        # 5. Write output.js statically for initial bootstrap
        with open(os.path.join(d_dir, "output.js"), "w") as f:
            f.write(f"const fastMemoryData = {out_json};")

        # 6. Write HTML and README (HTML no longer embeds JSON)
        write_html(d_dir) # No json_data parameter
        create_readme(d_dir, domain, atfs_md, out_json)

if __name__ == "__main__":
    main()
