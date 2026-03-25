#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;

pub mod parser;
pub mod cluster;

#[cfg(feature = "python")]
#[pyfunction]
fn process_markdown(text: String) -> PyResult<String> {
    let atfs = parser::parse_markdown(&text);
    
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

    let result_json = cluster::run_louvain(&edges, &atfs);
    Ok(result_json)
}

#[cfg(feature = "python")]
#[pymodule]
fn fastmemory(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_markdown, m)?)?;
    Ok(())
}
