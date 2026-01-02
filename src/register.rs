use once_cell::sync::Lazy;
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyKeyError};
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple, PyType};
use std::collections::HashMap;
use std::sync::Mutex;

create_exception!(cache_register, DuplicateRegisterEntry, PyException);
create_exception!(cache_register, InvalidObjectInRegister, PyException);

pub static GLOBAL_REGISTER: Lazy<Mutex<HashMap<String, HashMap<String, Py<PyAny>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[pyfunction]
pub fn clear_global_register() {
    let mut global = GLOBAL_REGISTER.lock().unwrap();
    global.clear();
}

#[pyfunction]
pub fn get_all_registers(py: Python<'_>) -> PyResult<Py<PyAny>> {
    // 1. Lock the global registry
    let global = GLOBAL_REGISTER.lock().unwrap();

    // 2. Create the main Python dictionary
    let result_dict = PyDict::new(py);

    // 3. Iterate over the Rust HashMap: RegisterName -> InnerMap
    for (reg_name, entries) in global.iter() {
        let sub_dict = PyDict::new(py);

        // 4. Iterate over the inner map: Key -> PythonObject
        for (key, obj) in entries.iter() {
            // Insert the object into the sub-dictionary.
            // We use clone_ref(py) to give the dictionary a new reference to the object.
            sub_dict.set_item(key, obj.clone_ref(py))?;
        }

        // Add the sub-dictionary to the main dictionary
        result_dict.set_item(reg_name, sub_dict)?;
    }

    // 5. Return the dictionary as a generic Python Object
    Ok(result_dict.into_any().unbind())
}

#[pyclass]
pub struct Register {
    #[pyo3(get)]
    name: String,
    expected_type: Option<Py<PyAny>>,
}

#[pymethods]
impl Register {
    #[new]
    #[pyo3(signature = (name, expected_type=None))]
    fn new(name: String, expected_type: Option<Bound<'_, PyType>>) -> Self {
        let mut global = GLOBAL_REGISTER.lock().unwrap();
        global.entry(name.clone()).or_insert_with(HashMap::new);

        Register {
            name,
            expected_type: expected_type.map(|t| t.into_any().unbind()),
        }
    }

    fn register(slf: PyRef<'_, Self>, py: Python<'_>, key: String) -> PyResult<Py<PyAny>> {
        let name = slf.name.clone();
        let expected_type = slf.expected_type.as_ref().map(|t| t.clone_ref(py));

        let decorator_logic = move |args: &Bound<'_, PyTuple>,
                                    _kwargs: Option<&Bound<'_, PyDict>>|
              -> PyResult<Py<PyAny>> {
            let obj = args.get_item(0)?;
            let py = obj.py();

            if let Some(ref expected) = expected_type {
                let expected_bound: &Bound<'_, PyAny> = expected.bind(py);
                if !obj.is_instance(expected_bound)? {
                    let type_name = expected_bound.repr()?.to_string_lossy().into_owned();
                    return Err(PyErr::new::<InvalidObjectInRegister, _>(format!(
                        "Attempted to register object '{}' in register '{}'. This register only accepts '{}' types.",
                        obj, name, type_name
                    )));
                }
            }

            let mut global = GLOBAL_REGISTER.lock().unwrap();
            let sub_register = global
                .get_mut(&name)
                .expect("Register name not initialized");

            if sub_register.contains_key(&key) {
                // This now works because create_exception! generates a proper exception type
                return Err(PyErr::new::<DuplicateRegisterEntry, _>(format!(
                    "Key '{}' already exists in register '{}'",
                    key, name
                )));
            }

            sub_register.insert(key.clone(), obj.clone().unbind());
            Ok(obj.clone().unbind())
        };

        let py_func = PyCFunction::new_closure(py, None, None, decorator_logic)?;
        Ok(py_func.into_any().unbind())
    }

    fn get(&self, py: Python<'_>, key: String) -> PyResult<Py<PyAny>> {
        let global = GLOBAL_REGISTER.lock().unwrap();
        let sub_register = global.get(&self.name).ok_or_else(|| {
            PyErr::new::<PyKeyError, _>(format!("Register '{}' not found", self.name))
        })?;

        match sub_register.get(&key) {
            Some(obj) => Ok(obj.clone_ref(py)),
            None => Ok(py.None()),
        }
    }
}
