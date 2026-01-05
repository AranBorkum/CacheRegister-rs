use crate::register::GLOBAL_REGISTER;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyModule};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

// Helper to skip hidden/heavy directories efficiently
fn is_ignored(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    if entry.file_type().is_dir() {
        return name.starts_with('.')
            || name == "__pycache__"
            || name == "node_modules"
            || name == "venv";
    }
    false
}

/// The core logic: Scans `base_path` for `target_filename` (ignoring .py extension) and imports found modules.
fn scan_and_import(py: Python<'_>, base_path: &str, target_filename: &str) -> PyResult<()> {
    // 1. Setup Python import tools
    let importlib = PyModule::import(py, "importlib")?;
    let sys = PyModule::import(py, "sys")?;

    // 2. Ensure base_path is in sys.path
    let path_list: Bound<'_, PyList> = sys.getattr("path")?.extract()?;
    let base_path_obj = base_path.into_pyobject(py)?;

    if !path_list.contains(&base_path_obj)? {
        path_list.insert(0, &base_path_obj)?;
    }

    let root = Path::new(base_path);

    // Normalize target: "registers.py" -> "registers", "registers" -> "registers"
    let target_stem = target_filename
        .strip_suffix(".py")
        .unwrap_or(target_filename);

    let walker = WalkDir::new(root).into_iter();

    // 3. Fast Traversal
    for entry in walker.filter_entry(|e| !is_ignored(e)) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Skip directories, we only care about files
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // 4. CHECK:
        // A) Does it have a .py extension?
        // B) Does the stem match our target?
        //    (e.g. file "registers.py" matches target "registers")
        let is_py_file = path.extension().map_or(false, |ext| ext == "py");
        let stems_match = path.file_stem().map_or(false, |s| s == target_stem);

        if is_py_file && stems_match {
            // 5. Convert file path to module path
            if let Ok(stripped) = path.strip_prefix(root) {
                let mut module_parts = Vec::new();
                for component in stripped.components() {
                    if let Some(s) = component.as_os_str().to_str() {
                        module_parts.push(s);
                    }
                }

                // Strip the ".py" extension from the last part for the import system
                if let Some(last) = module_parts.last_mut() {
                    if let Some(stripped_filename) = last.strip_suffix(".py") {
                        *last = stripped_filename;
                    }
                }

                let module_name = module_parts.join(".");

                // 6. Import the module
                if let Err(e) = importlib.call_method1("import_module", (module_name.clone(),)) {
                    eprintln!(
                        "Rust Autodiscover Error: Failed to import '{}': {}",
                        module_name, e
                    );
                }
            }
        }
    }
    Ok(())
}

/// Convenience wrapper: specifically looks for "registers.py"
#[pyfunction]
#[pyo3(signature = (base_path="."))]
pub fn autodiscover_registers(py: Python<'_>, base_path: &str) -> PyResult<()> {
    scan_and_import(py, base_path, "registers.py")
}

#[pyfunction]
#[pyo3(signature = (base_path="."))]
pub fn autoregister_registers(py: Python<'_>, base_path: &str) -> PyResult<()> {
    // 1. Acquire Lock & Collect Names
    // We use a block scope {} to ensure the lock is dropped immediately after we get the names.
    let register_names: Vec<String> = {
        let global = GLOBAL_REGISTER.lock().unwrap();
        // Clone the keys so we can own them outside the lock
        global.keys().cloned().collect()
    };
    // LOCK IS NOW RELEASED HERE automatically because 'global' went out of scope

    // 2. Perform Imports (Safe to re-acquire lock now)
    for reg_name in register_names {
        let module_name = format!("{}.py", reg_name);

        // This will call Python, which might call Rust, which might lock GLOBAL_REGISTER.
        // This is now safe because we aren't holding the lock anymore.
        scan_and_import(py, base_path, &module_name)?;
    }

    Ok(())
}
