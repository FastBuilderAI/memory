use std::fs;
use clap::{Parser, Subcommand};

use fastmemory::parser;
use fastmemory::cluster;

mod query;
mod server;
mod mcp;

#[derive(Parser)]
#[command(name = "fastmemory")]
#[command(about = "CBFDAE Ontology Clustering Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Optional fallback for `fastmemory <file.md>` default build
    file: Option<String>,

    /// Explicit local data path
    #[arg(long, global = true)]
    data: Option<String>,

    /// Enterprise remote data host URI (e.g. s3://bucket, postgres://db)
    #[arg(long, global = true)]
    datahost: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and build clustered JSON output
    Build {
        file: Option<String>,
    },
    /// Query the memory directly from the CLI
    Query {
        file: Option<String>,
        term: String,
    },
    /// Start the REST API server
    Serve {
        file: Option<String>,
        #[arg(short, long, default_value_t = 16743)]
        port: u16,
    },
    /// Run an MCP Server over Stdio
    Mcp {
        file: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    fastmemory::telemetry::LicenseTelemetry::ping().await;

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Build { file }) => {
            println!("{}", resolve_and_build(&cli, file.as_ref()));
        }
        Some(Commands::Query { file, term }) => {
            let json = resolve_and_build(&cli, file.as_ref());
            let result = query::search_memory(&json, term);
            println!("{}", result);
        }
        Some(Commands::Serve { file, port }) => {
            let json = resolve_and_build(&cli, file.as_ref());
            server::start_server(json, *port).await;
        }
        Some(Commands::Mcp { file }) => {
            let json = resolve_and_build(&cli, file.as_ref());
            mcp::start_mcp_server(json).await;
        }
        None => {
            if cli.file.is_some() || cli.data.is_some() || cli.datahost.is_some() {
                println!("{}", resolve_and_build(&cli, cli.file.as_ref()));
            } else {
                eprintln!("Usage: fastmemory <command> [args] [--data <path> | --datahost <uri>]");
                std::process::exit(1);
            }
        }
    }
}

fn resolve_and_build(cli: &Cli, cmd_file: Option<&String>) -> String {
    let content = if let Some(uri) = &cli.datahost {
        eprintln!("[Info] Fetching remote enterprise data from {}", uri);
        // Stub implementation for future remote integrations
        format!("## [ID: remote_stub]\n**Action:** Remote_Ingestion\n**Data_Connections:** {}\n**Access:** Role_Admin\n**Events:** remote_sync", uri)
    } else {
        let local_path = cli.data.as_ref()
            .or(cmd_file)
            .or(cli.file.as_ref())
            .expect("Error: Must provide a data source via <file>, --data, or --datahost.");
        
        fs::read_to_string(local_path).unwrap_or_else(|_| panic!("Unable to read file: {}", local_path))
    };

    let atfs = parser::parse_markdown(&content);
    
    let mut edges = Vec::new();
    for atf in &atfs {
        let f_id = format!("F_{}", atf.id);
        for link in &atf.data_connections { edges.push((f_id.clone(), format!("D_{}", link))); }
        for acc in atf.access.split(',') {
            let acc = acc.trim();
            if !acc.is_empty() { edges.push((f_id.clone(), format!("A_{}", acc))); }
        }
        for ev in atf.events.split(',') {
            let ev = ev.trim();
            if !ev.is_empty() { edges.push((f_id.clone(), format!("E_{}", ev))); }
        }
    }

    cluster::run_louvain(&edges, &atfs)
}
