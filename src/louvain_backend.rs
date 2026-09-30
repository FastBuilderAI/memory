//! Optional native clustering engine (FastBuilder.AI, proprietary), loaded at run time from its compiled library (C ABI).
//!
//! Where the library is loaded from (never the working directory or the platform's default search path, which can be
//! hijacked):
//!   1. `FASTMEMORY_NATIVE_LIB`, which must be an absolute path;
//!   2. the directory of the native image that contains this code (for bindings: the package's own library), which
//!      is where release packages bundle it;
//!   3. the directory of the running executable (for the CLI).
//! If none loads, `partition` returns `None`, the caller uses the inline Louvain, and `status()` says why.
//! `FASTMEMORY_CLUSTER=builtin` forces the inline Louvain.
//! Settings: `FASTMEMORY_NATIVE_RESOLUTION` (default 1.0) and `FASTMEMORY_NATIVE_MAX_LEVELS` (default 0 =
//! unlimited), the engine's own defaults.

use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::sync::OnceLock;

#[repr(C)]
struct FfiResult {
    partition_json: *mut c_char,
    community_count: usize,
    modularity: f64,
    levels: usize,
    elapsed_us: u64,
    error: *mut c_char,
}

type RunFn = unsafe extern "C" fn(*const c_char, f64, usize) -> FfiResult;
type FreeFn = unsafe extern "C" fn(*mut FfiResult);
type VersionFn = unsafe extern "C" fn() -> *const c_char;

struct Backend {
    lib: Option<Loaded>,
    /// Why no library is in use (empty when one is).
    reason: String,
}

struct Loaded {
    _lib: libloading::Library,
    run: RunFn,
    free: FreeFn,
    version: String,
    path: String,
}

fn lib_names() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &["libfastmemory_native.dylib"]
    } else if cfg!(target_os = "windows") {
        &["fastmemory_native.dll"]
    } else {
        &["libfastmemory_native.so"]
    }
}

/// Directory of the native image (shared library or executable) that contains this function.
#[cfg(unix)]
fn own_image_dir() -> Option<std::path::PathBuf> {
    #[repr(C)]
    struct DlInfo {
        dli_fname: *const c_char,
        dli_fbase: *mut std::ffi::c_void,
        dli_sname: *const c_char,
        dli_saddr: *mut std::ffi::c_void,
    }
    extern "C" {
        fn dladdr(addr: *const std::ffi::c_void, info: *mut DlInfo) -> i32;
    }
    let mut info = DlInfo { dli_fname: std::ptr::null(), dli_fbase: std::ptr::null_mut(), dli_sname: std::ptr::null(), dli_saddr: std::ptr::null_mut() };
    // SAFETY: dladdr only reads the address and fills `info`; dli_fname points into the loader's own storage.
    let ok = unsafe { dladdr(own_image_dir as *const std::ffi::c_void, &mut info) };
    if ok == 0 || info.dli_fname.is_null() {
        return None;
    }
    let p = std::path::PathBuf::from(unsafe { CStr::from_ptr(info.dli_fname) }.to_string_lossy().into_owned());
    let p = std::fs::canonicalize(&p).unwrap_or(p);
    p.parent().map(|d| d.to_path_buf())
}

#[cfg(not(unix))]
fn own_image_dir() -> Option<std::path::PathBuf> {
    None
}

/// Absolute candidate paths, in order, plus notes on anything skipped.
fn candidates() -> (Vec<std::path::PathBuf>, Vec<String>) {
    let (mut out, mut notes) = (Vec::new(), Vec::new());
    if let Some(p) = std::env::var("FASTMEMORY_NATIVE_LIB").ok().filter(|v| !v.trim().is_empty()) {
        let p = std::path::PathBuf::from(p);
        if p.is_absolute() {
            out.push(p);
        } else {
            notes.push(format!("FASTMEMORY_NATIVE_LIB ignored: not an absolute path ({})", p.display()));
        }
    }
    let exe_dir = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.to_path_buf()));
    for dir in [own_image_dir(), exe_dir].into_iter().flatten() {
        for n in lib_names() {
            let c = dir.join(n);
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    (out, notes)
}

fn load() -> Backend {
    if std::env::var("FASTMEMORY_CLUSTER").map(|v| v == "builtin").unwrap_or(false) {
        return Backend { lib: None, reason: "FASTMEMORY_CLUSTER=builtin".into() };
    }
    let (paths, mut notes) = candidates();
    for path in paths {
        if !path.is_file() {
            continue;
        }
        // SAFETY: loading a library runs its initialisers; the engine's are plain statics.
        let lib = match unsafe { libloading::Library::new(&path) } {
            Ok(l) => l,
            Err(e) => {
                notes.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        // SAFETY: the symbol types match the engine's C ABI (louvain_run, louvain_free_result, louvain_version).
        let syms = unsafe {
            (lib.get::<RunFn>(b"louvain_run\0"), lib.get::<FreeFn>(b"louvain_free_result\0"), lib.get::<VersionFn>(b"louvain_version\0"))
        };
        if let (Ok(run), Ok(free), Ok(version)) = syms {
            let (run, free, version) = (*run, *free, *version);
            // SAFETY: louvain_version returns a static NUL-terminated string.
            let v = unsafe { CStr::from_ptr(version()) }.to_string_lossy().into_owned();
            // quiet by default inside an embedding process (optional symbol: older builds lack it)
            if std::env::var("FASTMEMORY_NATIVE_VERBOSE").map_or(true, |v| v != "1") {
                // SAFETY: louvain_set_quiet(bool) only stores a process-wide flag.
                if let Ok(q) = unsafe { lib.get::<unsafe extern "C" fn(bool)>(b"louvain_set_quiet\0") } {
                    unsafe { q(true) };
                }
            }
            return Backend { lib: Some(Loaded { _lib: lib, run, free, version: v, path: path.display().to_string() }), reason: String::new() };
        }
        notes.push(format!("{}: missing native engine symbols", path.display()));
    }
    if std::env::var("FASTMEMORY_NATIVE_LIB").map_or(false, |v| !v.trim().is_empty()) {
        // an explicit request that failed is a misconfiguration: say so once
        eprintln!("fastmemory: native engine not loaded, using the inline Louvain ({})", notes.join("; "));
    }
    Backend { lib: None, reason: if notes.is_empty() { "no native engine library found".into() } else { notes.join("; ") } }
}

fn state() -> &'static Backend {
    static B: OnceLock<Backend> = OnceLock::new();
    B.get_or_init(load)
}

fn backend() -> Option<&'static Loaded> {
    state().lib.as_ref()
}

/// Which clustering backend is in use: `"native <version> (<path>)"` or `"inline"`.
pub fn describe() -> String {
    match backend() {
        Some(b) => format!("native {} ({})", b.version, b.path),
        None => "inline".to_string(),
    }
}

/// The backend and, when it is the inline Louvain, why: for provenance and stats.
pub fn status() -> serde_json::Value {
    match backend() {
        Some(b) => serde_json::json!({"backend": "native", "version": b.version, "path": b.path}),
        None => serde_json::json!({"backend": "inline", "reason": state().reason}),
    }
}

fn env_f64(k: &str, d: f64) -> f64 {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

/// Node -> community via the native engine, or `None` when the library is unavailable or reports an error.
pub fn partition(edges: &[(String, String)]) -> Option<HashMap<String, usize>> {
    let b = backend()?;
    let json = serde_json::to_string(&edges.iter().map(|(a, c)| [a.as_str(), c.as_str()]).collect::<Vec<_>>()).ok()?;
    let c = CString::new(json).ok()?;
    let resolution = env_f64("FASTMEMORY_NATIVE_RESOLUTION", 1.0);
    let max_levels = env_f64("FASTMEMORY_NATIVE_MAX_LEVELS", 0.0) as usize;
    // SAFETY: `c` is a valid NUL-terminated string that outlives the call; the result is freed exactly once below.
    let mut r = unsafe { (b.run)(c.as_ptr(), resolution, max_levels) };
    let out = if !r.error.is_null() || r.partition_json.is_null() {
        None
    } else {
        // SAFETY: partition_json is a NUL-terminated string owned by the result until it is freed.
        let s = unsafe { CStr::from_ptr(r.partition_json) }.to_string_lossy().into_owned();
        serde_json::from_str::<HashMap<String, usize>>(&s).ok()
    };
    // SAFETY: frees the strings allocated by louvain_run.
    unsafe { (b.free)(&mut r) };
    out
}
