#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;

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
        
        active_funcs = set()
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

    let result_json = cluster::run_louvain(&edges, &atfs_py);
    if let Some(handle) = ping_handle {
        let _ = handle.join();
    }
    Ok(result_json)
}

#[cfg(feature = "python")]
#[pymodule]
fn fastmemory(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_markdown, m)?)?;
    Ok(())
}
