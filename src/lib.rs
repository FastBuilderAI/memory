#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyList;

use std::env;

pub mod parser;
pub mod cluster;
pub mod telemetry;

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (text, concepts=None))]
fn process_markdown(py: Python, text: String, concepts: Option<Vec<String>>) -> PyResult<String> {
    let ping_handle = telemetry::LicenseTelemetry::ping();

    let python_code = cr#"
def extract_nltk_json(text, concepts):
    import re
    import json
    try:
        import nltk
    except ImportError:
        return '[]'
    try:
        nltk.data.find('tokenizers/punkt')
        nltk.data.find('taggers/averaged_perceptron_tagger_eng')
    except LookupError:
        nltk.download('punkt_tab', quiet=True)
        nltk.download('averaged_perceptron_tagger_eng', quiet=True)

    sentences = nltk.sent_tokenize(text)
    atfs = []
    
    concepts_lower = [c.lower() for c in (concepts or [])]
    
    for sent in sentences:
        tokens = nltk.word_tokenize(sent)
        pos_tags = nltk.pos_tag(tokens)
        
        # Inject all globally verified concepts as the root hub for the block
        active_funcs = set(concepts or [])
        for token in tokens:
            if token.lower() in concepts_lower:
                idx = concepts_lower.index(token.lower())
                active_funcs.add(concepts[idx])
                
        data = []
        access = []
        verbs = []
        
        for word, tag in pos_tags:
            # Preserve common chemical and logical punctuation, optionally scrub specific trailing commas
            word_clean = re.sub(r'[",.;]', '', word)
            
            # Allow items of length >= 1 (to capture atomic symbols like 'O', 'Cl', 'Fe') 
            # but ensure they are not just empty or spaces
            if not word_clean or len(word_clean) < 1: continue
            if word_clean.lower() in concepts_lower: continue
            
            if tag.startswith('NN'): data.append(word_clean.capitalize())
            elif tag.startswith('JJ'): access.append(word_clean.capitalize())
            elif tag.startswith('VB'): verbs.append(word_clean.capitalize())
            
        final_funcs = list(active_funcs) if active_funcs else verbs
        if not final_funcs and data:
            final_funcs = [data[0] + "Controller"]
            
        for f in set(final_funcs):
            atfs.append({
                "id": f,
                "action": "Concept" if active_funcs else "Extrapolated",
                "input": "",
                "logic": "",
                "data_connections": list(set(data)),
                "access": ",".join(set(access)),
                "events": ""
            })
            
    return json.dumps(atfs)
"#;

    let module = PyModule::from_code(py, python_code, c"extractor.py", c"extractor")?;
    let concepts_list = concepts.unwrap_or_default();
    let atfs_json: String = module.getattr("extract_nltk_json")?.call1((text, concepts_list))?.extract()?;
    let atfs_py: Vec<parser::Atf> = serde_json::from_str(&atfs_json).unwrap_or_default();

    let mut edges = Vec::new();
    for atf in &atfs_py {
        let f_id = format!("F_{}", atf.id);
        for link in &atf.data_connections { 
            edges.push((f_id.clone(), format!("D_{}", link))); 
        }
        for acc in atf.access.split(',') {
            let acc = acc.trim();
            if !acc.is_empty() { 
                edges.push((f_id.clone(), format!("A_{}", acc))); 
            }
        }
        for ev in atf.events.split(',') {
            let ev = ev.trim();
            if !ev.is_empty() { 
                edges.push((f_id.clone(), format!("E_{}", ev))); 
            }
        }
    }

    let result_json = cluster::run_louvain_inline(&edges, &atfs_py);
    if let Some(handle) = ping_handle {
        let _ = handle.join();
    }
    Ok(result_json)
}

/// Internal: extract ATFs from text using NLTK (pure Python, no Louvain)
const NLTK_EXTRACT_CODE: &str = r#"
import re, json

def _ensure_nltk():
    import nltk
    try:
        nltk.data.find('tokenizers/punkt')
        nltk.data.find('taggers/averaged_perceptron_tagger_eng')
    except LookupError:
        nltk.download('punkt_tab', quiet=True)
        nltk.download('averaged_perceptron_tagger_eng', quiet=True)

def _extract_one(text, concepts):
    import nltk
    sentences = nltk.sent_tokenize(text)
    atfs = []
    concepts_lower = [c.lower() for c in (concepts or [])]
    for sent in sentences:
        tokens = nltk.word_tokenize(sent)
        pos_tags = nltk.pos_tag(tokens)
        # Inject global multi-concept hubs
        active_funcs = set(concepts or [])
        for token in tokens:
            if token.lower() in concepts_lower:
                idx = concepts_lower.index(token.lower())
                active_funcs.add(concepts[idx])
        data, access, verbs = [], [], []
        for word, tag in pos_tags:
            word_clean = re.sub(r'[",.;]', '', word)
            if not word_clean or len(word_clean) < 1: continue
            if word_clean.lower() in concepts_lower: continue
            if tag.startswith('NN'): data.append(word_clean.capitalize())
            elif tag.startswith('JJ'): access.append(word_clean.capitalize())
            elif tag.startswith('VB'): verbs.append(word_clean.capitalize())
        final_funcs = list(active_funcs) if active_funcs else verbs
        if not final_funcs and data:
            final_funcs = [data[0] + "Controller"]
        for f in set(final_funcs):
            atfs.append({"id": f, "action": "Concept" if active_funcs else "Extrapolated",
                         "input": "", "logic": "", "data_connections": list(set(data)),
                         "access": ",".join(set(access)), "events": ""})
    return json.dumps(atfs)

def _worker_init():
    import nltk
    _ensure_nltk()

def _process_item(args):
    doc_id, text, concepts = args
    try:
        result = _extract_one(text, concepts)
        return (doc_id, result, None)
    except Exception as e:
        return (doc_id, '[]', str(e))
"#;

/// Rust-side function to run Louvain on extracted ATFs
fn run_louvain_for_atfs(atfs_json: &str) -> String {
    let atfs_py: Vec<parser::Atf> = serde_json::from_str(atfs_json).unwrap_or_default();
    let mut edges = Vec::new();
    for atf in &atfs_py {
        let f_id = format!("F_{}", atf.id);
        for link in &atf.data_connections {
            edges.push((f_id.clone(), format!("D_{}", link)));
        }
        for acc in atf.access.split(',') {
            let acc = acc.trim();
            if !acc.is_empty() {
                edges.push((f_id.clone(), format!("A_{}", acc)));
            }
        }
        for ev in atf.events.split(',') {
            let ev = ev.trim();
            if !ev.is_empty() {
                edges.push((f_id.clone(), format!("E_{}", ev)));
            }
        }
    }
    cluster::run_louvain_inline(&edges, &atfs_py)
}

/// Process a batch of documents in parallel using Python multiprocessing for NLTK,
/// then Rust Louvain clustering for each result.
///
/// Args:
///   documents: List of (doc_id, content, concepts_list) tuples
///   workers: Optional number of parallel workers (default: FASTMEMORY_WORKERS env or CPU cores)
///
/// Returns:
///   List of (doc_id, result_json, error_or_none) tuples
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (documents, workers=None))]
fn process_batch(py: Python, documents: Vec<(String, String, Vec<String>)>, workers: Option<usize>) -> PyResult<Vec<(String, String, Option<String>)>> {
    let _ping_handle = telemetry::LicenseTelemetry::ping();

    // Determine worker count: argument > env > CPU cores
    let num_workers = workers.unwrap_or_else(|| {
        env::var("FASTMEMORY_WORKERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|p| p.get())
                    .unwrap_or(4)
            })
    });

    let total = documents.len();
    eprintln!("[FastMemory] Batch: {} documents, {} workers", total, num_workers);

    // Write worker code to a temp file so multiprocessing spawn can import it
    let worker_code = format!(r#"
{extract_code}

import json
from concurrent.futures import ProcessPoolExecutor, as_completed

def run_batch(docs, num_workers):
    _ensure_nltk()
    results = [None] * len(docs)
    with ProcessPoolExecutor(max_workers=num_workers, initializer=_worker_init) as executor:
        future_to_idx = {{}}
        for i, (doc_id, text, concepts) in enumerate(docs):
            fut = executor.submit(_process_item, (doc_id, text, concepts))
            future_to_idx[fut] = i
        done = 0
        for future in as_completed(future_to_idx):
            idx = future_to_idx[future]
            try:
                results[idx] = future.result()
            except Exception as e:
                doc_id = docs[idx][0]
                results[idx] = (doc_id, '[]', str(e))
            done += 1
            if done % 1000 == 0:
                print(f'[FastMemory] NLTK extracted {{done}}/{{len(docs)}}...', flush=True)
    return results
"#, extract_code = NLTK_EXTRACT_CODE);

    // Write to temp file, import it, then clean up
    let launcher_code = cr#"
import tempfile, importlib, sys, os

def launch_batch(worker_code_str, docs, num_workers):
    tmpdir = tempfile.mkdtemp(prefix='fastmem_')
    worker_path = os.path.join(tmpdir, '_fm_batch_worker.py')
    with open(worker_path, 'w') as f:
        f.write(worker_code_str)
    sys.path.insert(0, tmpdir)
    try:
        mod = importlib.import_module('_fm_batch_worker')
        return mod.run_batch(docs, num_workers)
    finally:
        sys.path.remove(tmpdir)
        try:
            os.remove(worker_path)
            os.rmdir(tmpdir)
        except:
            pass
"#;

    let launcher = PyModule::from_code(py, launcher_code, c"launcher.py", c"launcher")?;
    let py_docs = PyList::new(py, documents.iter().map(|(id, content, concepts)| {
        (id.as_str(), content.as_str(), concepts.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }))?;

    let nltk_results: Vec<(String, String, Option<String>)> = launcher
        .getattr("launch_batch")?
        .call1((&worker_code, py_docs, num_workers))?
        .extract()?;

    eprintln!("[FastMemory] NLTK extraction complete, running Louvain clustering...");

    // Now run Rust Louvain INLINE on each result (no subprocess!)
    let mut final_results: Vec<(String, String, Option<String>)> = Vec::with_capacity(total);
    for (i, (doc_id, atfs_json, error)) in nltk_results.into_iter().enumerate() {
        if error.is_some() {
            final_results.push((doc_id, "[]".to_string(), error));
            continue;
        }
        let clustered = run_louvain_for_atfs(&atfs_json);
        final_results.push((doc_id, clustered, None));

        if (i + 1) % 1000 == 0 {
            eprintln!("[FastMemory] Louvain clustered {}/{}", i + 1, total);
        }
    }

    eprintln!("[FastMemory] Batch complete: {} documents processed", total);
    Ok(final_results)
}

/// Run Louvain clustering on ATFs JSON directly from Python.
/// This is the fast path — pure Rust, no subprocess.
///
/// Args:
///   atfs_json: JSON string of ATF array from NLTK extraction
///
/// Returns:
///   JSON string of clustered topology graph
#[cfg(feature = "python")]
#[pyfunction]
fn louvain_cluster(atfs_json: String) -> PyResult<String> {
    Ok(run_louvain_for_atfs(&atfs_json))
}

/// Return the NLTK extraction Python source code.
/// Used by external drivers to create persistent worker pools.
#[cfg(feature = "python")]
#[pyfunction]
fn get_nltk_extract_code() -> PyResult<String> {
    Ok(NLTK_EXTRACT_CODE.to_string())
}

/// Get the default worker count (FASTMEMORY_WORKERS env or CPU cores)
#[cfg(feature = "python")]
#[pyfunction]
fn get_worker_count() -> PyResult<usize> {
    Ok(env::var("FASTMEMORY_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4)
        }))
}

#[cfg(feature = "python")]
#[pymodule]
fn fastmemory(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(process_batch, m)?)?;
    m.add_function(wrap_pyfunction!(louvain_cluster, m)?)?;
    m.add_function(wrap_pyfunction!(get_nltk_extract_code, m)?)?;
    m.add_function(wrap_pyfunction!(get_worker_count, m)?)?;
    Ok(())
}

