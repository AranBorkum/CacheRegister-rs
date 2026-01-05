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
    let global = GLOBAL_REGISTER.lock().unwrap();
    let result_dict = PyDict::new(py);

    for (reg_name, entries) in global.iter() {
        let sub_dict = PyDict::new(py);
        for (key, obj) in entries.iter() {
            sub_dict.set_item(key, obj.clone_ref(py))?;
        }
        result_dict.set_item(reg_name, sub_dict)?;
    }
    Ok(result_dict.into_any().unbind())
}

// --- NEW HELPER STRUCT ---
#[pyclass]
struct RegisterFactory {
    expected_type: Py<PyAny>,
}

#[pymethods]
impl RegisterFactory {
    #[pyo3(signature = (name))]
    fn __call__(&self, py: Python<'_>, name: String) -> PyResult<Register> {
        // Initialize global state
        let mut global = GLOBAL_REGISTER.lock().unwrap();
        global.entry(name.clone()).or_insert_with(HashMap::new);

        // Create the Register with the constraints from the factory
        Ok(Register {
            name,
            expected_type: Some(self.expected_type.clone_ref(py)),
        })
    }
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
                let expected_bound = expected.bind(py);

                // 1. Strict Check: Is it an instance? (e.g. A())
                let mut is_valid = obj.is_instance(expected_bound)?;

                // 2. Permissive Check: If it's a class, is it a subclass? (e.g. class B(A))
                // Only run this if the instance check failed.
                if !is_valid {
                    // Check if the object being registered is itself a Type (Class)
                    if let Ok(obj_as_type) = obj.cast::<PyType>() {
                        // We use Python's built-in issubclass checks
                        // (handles tuples of types in 'expected' correctly)
                        let builtins = PyModule::import(py, "builtins")?;

                        // We wrap this in Result in case 'expected' is not a valid class (e.g. 5)
                        // which would cause issubclass to raise a TypeError.
                        if let Ok(res) =
                            builtins.call_method1("issubclass", (obj_as_type, expected_bound))
                        {
                            if res.is_truthy()? {
                                is_valid = true;
                            }
                        }
                    }
                }

                if !is_valid {
                    let type_name = expected_bound.repr()?.to_string_lossy().into_owned();
                    let obj_repr = obj.repr()?.to_string_lossy().into_owned();

                    return Err(PyErr::new::<InvalidObjectInRegister, _>(format!(
                        "Attempted to register object '{}' in register '{}'. This register only accepts '{}' types or subclasses.",
                        obj_repr, name, type_name
                    )));
                }
            }

            // ... (rest of the function: duplicate check and insert) ...
            let mut global = GLOBAL_REGISTER.lock().unwrap();
            let sub_register = global
                .get_mut(&name)
                .expect("Register name not initialized");

            if sub_register.contains_key(&key) {
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

    // --- MODIFIED CLASS GETITEM ---
    #[classmethod]
    fn __class_getitem__(
        _cls: &Bound<'_, PyType>,
        item: &Bound<'_, PyAny>,
    ) -> PyResult<RegisterFactory> {
        // Return the factory that captures the type 'item' (e.g., int, MyClass)
        Ok(RegisterFactory {
            expected_type: item.clone().unbind(),
        })
    }
}
