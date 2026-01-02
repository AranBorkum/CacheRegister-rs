use pyo3::prelude::*;

mod autodiscovery;
mod register;

// --- Module Definition ---
#[pymodule]
fn _cache_register(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<register::Register>()?;
    m.add(
        "DuplicateRegisterEntry",
        py.get_type::<register::DuplicateRegisterEntry>(),
    )?;
    m.add(
        "InvalidObjectInRegister",
        py.get_type::<register::InvalidObjectInRegister>(),
    )?;
    m.add_function(wrap_pyfunction!(register::clear_global_register, m)?)?;
    m.add_function(wrap_pyfunction!(register::get_all_registers, m)?)?;
    m.add_function(wrap_pyfunction!(autodiscovery::autodiscover_registers, m)?)?;
    m.add_function(wrap_pyfunction!(autodiscovery::autoregister_registers, m)?)?;

    Ok(())
}
