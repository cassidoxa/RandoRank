use pyo3::create_exception;
use pyo3::prelude::*;

mod glicko;
mod math;

#[pymodule]
fn randorank(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<glicko::MultiPeriod>()?;
    m.add("GlickoError", py.get_type::<GlickoError>())?;
    Ok(())
}

create_exception!(randorank, GlickoError, pyo3::exceptions::Exception);
