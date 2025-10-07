//! # MRP FFI
//!
//! Python 綁定層（PyO3）

use pyo3::prelude::*;

pub mod python;

/// Python 模組註冊
#[pymodule]
fn mrp_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<python::PyMrpCalculator>()?;
    m.add_class::<python::PyDemand>()?;
    m.add_class::<python::PySupply>()?;
    m.add_class::<python::PyInventory>()?;
    m.add_class::<python::PyMrpConfig>()?;
    Ok(())
}
