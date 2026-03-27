# FastMemory

**FastMemory** is an ontological clustering engine that transforms flat, unstructured text embeddings into a structured, agent-navigable functional memory graph using the **CBFDAE** (Component, Block, Function, Data, Access, Event) taxonomy.

Developed by [FastBuilder.AI](https://fastbuilder.ai), FastMemory bridges the gap between shallow vector retrieval (RAG) and deterministic computational memory.

## 🤬 Developer Pain Points & The FastMemory Solution

Building reliable AI agents on top of massive Enterprise codebases and datasets is incredibly hard. FastMemory directly solves the three biggest pain points developers face today:

1. **RAG Hallucinations**: Standard vector similarity retrieves unrelated text chunks just because they share keywords (e.g., retrieving the "Login Code" when the user asked about the "Login Bug Ticket"). FastMemory provides **Deterministic Pathfinding** through isolated functional clusters.
2. **Context Fragmentation**: Naive text chunking destroys logical boundaries, losing the surrounding context of a function. FastMemory parses semantic topologies into grouped **Cognitive Blocks**, providing the AI with sibling functions and deterministic access restrictions.
3. **Graph DB Sync Overhead**: Piping hierarchical data into Neo4J normally requires complex, fragile NLP and ETL pipelines. The FastMemory Rust engine does this natively in milliseconds using structural Louvain clustering.

---

## 🗺️ The Google Maps Analogy

Imagine opening Google Maps, but all you can see are roads and paths. There are no building names, no entry gates, no transaction information for the buildings, and no communication routing.

If you asked a humanoid robot to navigate to a hospital using this map, it would only see a "road to a doctor", a "road to a bed", a "road to a nurse", and a "road to a pharmacy." It would have a profoundly hard time knowing the modes and modality of how to actually behave, act, and pursue every target differently depending on context.

**That is exactly what happens when you use standard RAG, semantic ontologies, or flat vector graphs.**

|                   Standard Ontology / RAG                   |                      FastMemory CBFDAE Map                       |
| :---------------------------------------------------------: | :--------------------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/analogies/roads_only.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/analogies/structured_city.png" width="100%" /> |

You simply have node-to-node semantic edges. You possess the "roads" (cosine similarity), but you lack the "buildings" (Functional Components) and the "rules of entry/engagement" (Access and Events).

**FastMemory** solves this. We utilize high-speed Community Detection (Louvain clustering) to mathematically derive and enhance this network of data for **direct AI usage**, translating raw text into executable cognitive blocks.

---

## 🔍 Features & Benefits

- **CBFDAE Ontology**: Information isn't just stored; it is classified into **C**omponents, **B**locks, **F**unctions, **D**ata, **A**ccess restrictions, and **E**vents.
- **Deterministic Pathfinding**: Eliminates RAG hallucinations. An AI doesn't "guess" the answer based on semantic proximity; it traverses a rigorous, rule-based logic graph.
- **The Agentic Query Engine**: Deep recursive subtree targeting. When you query FastMemory, it doesn't just return a matching string—it returns the _deepest logical encompassing Block_, providing the AI with sibling functions and contextual boundaries.
- **Enterprise Native**: Designed to sit on top of Datawarehouses, SAP, Databricks, AWS Glue, and Fabric.

---

## 📊 Before & After FastMemory

Standard vector RAG databases index chunks individually, often losing the multi-hop reasoning capability required to trace dependencies. FastMemory restructures these into event-driven, hierarchical memory blocks.

_(You can open the interactive D3.js visualizations directly in your browser from the `example/` directories!)_

### 🏥 Health Science

|                Before: Flat Semantic Vectors                 |          After: Clustered Functional Memory Graph           |
| :----------------------------------------------------------: | :---------------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/health_science/before.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/health_science/after.png" width="100%" /> |

### 🤖 Robotics

|             Before: Flat Semantic Vectors              |       After: Clustered Functional Memory Graph        |
| :----------------------------------------------------: | :---------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/robotics/before.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/robotics/after.png" width="100%" /> |

### 🚗 Driverless Cars

|                 Before: Flat Semantic Vectors                 |           After: Clustered Functional Memory Graph           |
| :-----------------------------------------------------------: | :----------------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/driverless_cars/before.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/driverless_cars/after.png" width="100%" /> |

### 📈 Business Analytics

|                  Before: Flat Semantic Vectors                   |            After: Clustered Functional Memory Graph             |
| :--------------------------------------------------------------: | :-------------------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/business_analytics/before.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/business_analytics/after.png" width="100%" /> |

### ✉️ Email Analysis

|                Before: Flat Semantic Vectors                 |          After: Clustered Functional Memory Graph           |
| :----------------------------------------------------------: | :---------------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/email_analysis/before.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/email_analysis/after.png" width="100%" /> |

### 📋 Audit Operations

|            Before: Flat Semantic Vectors            |      After: Clustered Functional Memory Graph      |
| :-------------------------------------------------: | :------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/audit/before.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/audit/after.png" width="100%" /> |

### 🌍 World Events

|               Before: Flat Semantic Vectors                |         After: Clustered Functional Memory Graph          |
| :--------------------------------------------------------: | :-------------------------------------------------------: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/world_events/before.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/world_events/after.png" width="100%" /> |

---

## 📦 Installation

**Rust (Cargo) - CLI Utility**
To install the standalone `fastmemory` CLI tool for terminal, server, or MCP usage:
```bash
cargo install fastmemory
```

**Python (PyPI) - Native Import**
To install the high-speed Python module (built natively via PyO3) for direct integration into your Python AI applications:
```bash
pip install fastmemory
```

---

## 🚀 Usage Guide

FastMemory can be utilized natively from the command line, spun up as an enterprise REST server, or imported directly into your Python scripts.

### 1. Terminal CLI (via Cargo)

```bash
# Build the memory graph from an ATF Markdown file
$ fastmemory build data/input.md

# Instantly query the hierarchical graph
$ fastmemory query data/input.md "reimbursement"
```

### 2. Python (Direct Import)

By utilizing our `pip` module, your Python loops can pass markdown directly to the compiled Rust engine without any JSON/CLI overhead. The resulting graph JSON is computed instantly via Louvain community detection.

```python
import fastmemory

# 1. Define or fetch your raw Action-Topology Format (ATF) text
markdown_text = """
## [ID: auth_module]
**Action:** Validate_Token
**Data_Connections:** session_uuid
**Access:** Role_Admin
**Events:** User_Login
"""

# 2. Pass strings synchronously into the Rust engine
cbfdae_json_graph = fastmemory.process_markdown(markdown_text)

print(cbfdae_json_graph)
```

### 3. Running as an Enterprise Service

FastMemory ships with a highly optimized embedded Axum web server and MCP (Model Context Protocol) integration for AI agents:

```bash
# Boot the REST API locally or enterprise-wide
$ fastmemory serve data/input.md --port 16743
# Query: curl http://localhost:16743/query?q=reimbursement

# Boot standard Stdio MCP (for Claude / Gemini IDE integration)
$ fastmemory mcp data/input.md
```

### 4. Enterprise Data Ingestion

In addition to positional file arguments, FastMemory supports enterprise-grade ingestion strings for dynamic pipelines:

- **Local Directories (`--data`)**: Pass a local directory to parse and cluster multiple ATF Markdown files continuously.
  ```bash
  $ fastmemory build --data /var/lib/fastmemory/data
  ```
- **Remote Pipelines (`--datahost`)**: Bind directly to Data Warehouses (Snowflake, BigQuery), Data Lakes (Databricks, Fabric), or S3 by passing a connection URI. FastMemory will securely intercept the URI and dynamically ingest the remote structures.
  ```bash
  $ fastmemory serve --port 16743 --datahost postgres://db_user:secret@localhost:5432/app
  $ fastmemory mcp --datahost s3://corporate-bucket/atfs/
  ```

> [!TIP]
> **Large Scale Graph DB Memory**: When scaling FastMemory beyond local processing, the clustered JSON output explicitly maps into systems like **Neo4J**. FastMemory naturally supports partial topology updates. Please read our **[Production Scaling & Graph DB Ingestion Guide](production.md)** for detailed Python and Cypher deployment patterns.

### 5. Advanced Security & Federated Auth

Data access within FastMemory is rigorously secured at the graph layer. Utilizing the **`A_` (Access)** node topology, you can map federated IAM rules (like AWS IAM or Azure AD) directly onto specific memory blocks.

- **Wrapper Implementation**: Place an API Gateway ahead of `fastmemory serve` to enforce standard OAuth/SAML.
- **Code-Level Auth**: AI agents parsing the memory graph will inherently see the `A_Role_Admin` nodes attached to functions, allowing the agent to deterministically self-regulate access before taking action.

---

## 🧠 Applications

| Standard RAG Robot Brain | FastMemory CBFDAE Robot Brain |
| :---: | :---: |
| <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/analogies/robot_rag_memory.png" width="100%" /> | <img src="https://raw.githubusercontent.com/FastBuilderAI/memory/main/example/analogies/robot_fast_memory.png" width="100%" /> |

- **Agentic Apps & SaaS**: Integrate `fastmemory mcp` directly into your proprietary AI loops. Instead of sending agents to vector DBs, send them into a FastMemory graph where they can extract isolated, functional context blocks to execute SaaS workflows.
- **Fast Software Engineering**: In [FastBuilder.AI](https://fastbuilder.ai), FastMemory acts as the structural brain for rapid feature development. By indexing the entire application architecture into an ontological graph, coding agents can query precisely how a proposed change will impact distant, decoupled components.
- **The Possibilities are Endless**: Medical diagnostics routing, autonomous drone navigation logic, enterprise compliance auditing, etc.

---

## 📄 License & Commercial Terms

This project is licensed under the **MIT License**.

**COMMERCIAL USE EXCEPTION**
Notwithstanding the MIT License, any commercial entity, company, or organization that has generated gross revenue exceeding **USD $20,000,000** in its most recently completed fiscal year or current interim year must obtain an explicit commercial license from FastBuilder.AI prior to using, copying, modifying, merging, publishing, distributing, sublicensing, or selling copies of this Software.

> [!NOTE]
> **When is the Commercial License Necessary?**
> For standard projects involving small or moderate numbers of documents, any basic Python script or standard semantic dictionary will suffice. FastMemory, however, is a **hyper-optimized, concurrent Rust Graph Processing Engine**. If your enterprise is parsing, isolating, and synchronizing **millions of documents** into systems like Neo4J and executing real-time partial updates, this engine is designed for your scale. The commercial license ensures FastBuilder.AI can continue maintaining and pushing the limits of this high-performance architecture.
