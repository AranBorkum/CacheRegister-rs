use pyo3::prelude::*;
use pyo3::types::{PyList, PyModule};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use crate::register::GLOBAL_REGISTER;

fn is_ignored(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    if entry.file_type().is_dir() {
        return name.starts_with('.')
            || name == "__pycache__"
            || name == "node_modules"
            || name == "venv"
            || name == "env";
    }
    false
}

/// Helper: Checks if a directory is a valid Python package (has __init__.py)
fn is_python_package(dir_path: &Path) -> bool {
    dir_path.join("__init__.py").exists()
}

fn scan_and_import(py: Python<'_>, base_path: &str, target_filename: &str) -> PyResult<()> {
    let importlib = PyModule::import(py, "importlib")?;
    let sys = PyModule::import(py, "sys")?;

    let path_list: Bound<'_, PyList> = sys.getattr("path")?.extract()?;
    let base_path_obj = base_path.into_pyobject(py)?;

    if !path_list.contains(&base_path_obj)? {
        path_list.insert(0, &base_path_obj)?;
    }

    let root = Path::new(base_path);

    // Normalize target: "registers.py" -> "registers"
    let target_stem = target_filename
        .strip_suffix(".py")
        .unwrap_or(target_filename);
    // Construct the expected file name: "registers.py"
    let target_file_name = format!("{}.py", target_stem);

    let walker = WalkDir::new(root).into_iter();

    for entry in walker.filter_entry(|e| !is_ignored(e)) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let file_name = entry.file_name().to_str().unwrap_or("");

        // --- LOGIC: Determine if this entry is a match ---
        let mut is_match = false;

        // Case 1: File Match (e.g., "registers.py")
        if entry.file_type().is_file() && file_name == target_file_name {
            is_match = true;
        }
        // Case 2: Package Match (e.g., directory "registers/" containing "__init__.py")
        else if entry.file_type().is_dir() && file_name == target_stem {
            if is_python_package(path) {
                is_match = true;
            }
        }

        if is_match {
            // Calculate module path
            if let Ok(stripped) = path.strip_prefix(root) {
                let mut module_parts = Vec::new();
                for component in stripped.components() {
                    if let Some(s) = component.as_os_str().to_str() {
                        module_parts.push(s);
                    }
                }

                // If it's a file, we must strip the ".py" extension from the last component
                // If it's a directory, the directory name IS the module name, so we leave it alone.
                if entry.file_type().is_file() {
                    if let Some(last) = module_parts.last_mut() {
                        if let Some(stripped_filename) = last.strip_suffix(".py") {
                            *last = stripped_filename;
                        }
                    }
                }

                let module_name = module_parts.join(".");

                // Import
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
