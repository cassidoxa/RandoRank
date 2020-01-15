use pyo3::create_exception;
use pyo3::prelude::*;

mod glicko;

#[pymodule]
fn randorank(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<glicko::Period>()?;
    Ok(())
}

create_exception!(randorank, GlickoError, pyo3::exceptions::Exception);