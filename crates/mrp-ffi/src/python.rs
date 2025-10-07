//! Python 綁定實現

use pyo3::prelude::*;

/// Python MRP 計算器
#[pyclass(name = "MrpCalculator")]
pub struct PyMrpCalculator {
    // TODO: 內部狀態
}

#[pymethods]
impl PyMrpCalculator {
    #[new]
    fn new() -> Self {
        Self {}
    }

    /// 執行 MRP 計算
    fn calculate(&self) -> PyResult<String> {
        Ok("MRP calculation result (TODO)".to_string())
    }
}

/// Python 需求
#[pyclass(name = "Demand")]
pub struct PyDemand {
    // TODO: 字段
}

#[pymethods]
impl PyDemand {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

/// Python 供應
#[pyclass(name = "Supply")]
pub struct PySupply {
    // TODO: 字段
}

#[pymethods]
impl PySupply {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

/// Python 庫存
#[pyclass(name = "Inventory")]
pub struct PyInventory {
    // TODO: 字段
}

#[pymethods]
impl PyInventory {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

/// Python MRP 配置
#[pyclass(name = "MrpConfig")]
pub struct PyMrpConfig {
    #[pyo3(get, set)]
    pub component_id: String,
    #[pyo3(get, set)]
    pub lead_time_days: u32,
    #[pyo3(get, set)]
    pub procurement_type: String, // "Make" or "Buy"
    #[pyo3(get, set)]
    pub lot_sizing_rule: String, // "LotForLot", "FOQ", "EOQ", "POQ", "MinMax"
    #[pyo3(get, set)]
    pub fixed_lot_size: Option<f64>,
    #[pyo3(get, set)]
    pub minimum_order_qty: Option<f64>,
    #[pyo3(get, set)]
    pub maximum_order_qty: Option<f64>,
    #[pyo3(get, set)]
    pub order_multiple: Option<f64>,
    #[pyo3(get, set)]
    pub safety_stock: f64,
    #[pyo3(get, set)]
    pub planning_horizon_days: u32,
    #[pyo3(get, set)]
    pub allow_negative_inventory: bool,
}

#[pymethods]
impl PyMrpConfig {
    #[new]
    #[pyo3(signature = (component_id, lead_time_days, procurement_type="Make", allow_negative_inventory=false))]
    fn new(
        component_id: String,
        lead_time_days: u32,
        procurement_type: &str,
        allow_negative_inventory: bool,
    ) -> Self {
        Self {
            component_id,
            lead_time_days,
            procurement_type: procurement_type.to_string(),
            lot_sizing_rule: "LotForLot".to_string(),
            fixed_lot_size: None,
            minimum_order_qty: None,
            maximum_order_qty: None,
            order_multiple: None,
            safety_stock: 0.0,
            planning_horizon_days: 90,
            allow_negative_inventory,
        }
    }
}

/// 內部方法實現（不暴露給 Python）
impl PyMrpConfig {
    /// 轉換為 Rust MrpConfig（內部使用）
    pub(crate) fn to_rust_config(&self) -> PyResult<mrp_core::MrpConfig> {
        use mrp_core::{LotSizingRule, MrpConfig, ProcurementType};
        use rust_decimal::Decimal;

        let procurement_type = match self.procurement_type.as_str() {
            "Make" => ProcurementType::Make,
            "Buy" => ProcurementType::Buy,
            "Transfer" => ProcurementType::Transfer,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid procurement_type: {}, must be 'Make', 'Buy', or 'Transfer'",
                    self.procurement_type
                )))
            }
        };

        let lot_sizing_rule = match self.lot_sizing_rule.as_str() {
            "LotForLot" => LotSizingRule::LotForLot,
            "FOQ" | "FixedOrderQuantity" => LotSizingRule::FixedOrderQuantity,
            "EOQ" | "EconomicOrderQuantity" => LotSizingRule::EconomicOrderQuantity,
            "POQ" | "PeriodOrderQuantity" => LotSizingRule::PeriodOrderQuantity,
            "MinMax" => LotSizingRule::MinMax,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid lot_sizing_rule: {}",
                    self.lot_sizing_rule
                )))
            }
        };

        let mut config = MrpConfig::new(self.component_id.clone(), self.lead_time_days, procurement_type)
            .with_lot_sizing_rule(lot_sizing_rule)
            .with_safety_stock(Decimal::try_from(self.safety_stock).unwrap_or_default())
            .with_planning_horizon(self.planning_horizon_days)
            .with_allow_negative_inventory(self.allow_negative_inventory);

        if let Some(size) = self.fixed_lot_size {
            config = config.with_fixed_lot_size(Decimal::try_from(size).unwrap_or_default());
        }
        if let Some(min) = self.minimum_order_qty {
            config = config.with_minimum_order_qty(Decimal::try_from(min).unwrap_or_default());
        }
        if let Some(max) = self.maximum_order_qty {
            config = config.with_maximum_order_qty(Decimal::try_from(max).unwrap_or_default());
        }
        if let Some(multiple) = self.order_multiple {
            config = config.with_order_multiple(Decimal::try_from(multiple).unwrap_or_default());
        }

        Ok(config)
    }
}
