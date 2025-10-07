//! # MRP Calculation Engine
//!
//! 核心 MRP 計算引擎

pub mod bucketing;
pub mod calculator;
pub mod lead_time;
pub mod lot_sizing;
pub mod netting;
pub mod pegging;

// Re-export 主要類型
pub use calculator::MrpCalculator;
pub use netting::NetRequirement;

/// MRP 計算結果
#[derive(Debug, Clone)]
pub struct MrpResult {
    /// 計劃訂單
    pub planned_orders: Vec<mrp_core::PlannedOrder>,

    /// 需求追溯
    pub pegging: std::collections::HashMap<uuid::Uuid, Vec<mrp_core::PeggingRecord>>,

    /// 警告信息
    pub warnings: Vec<MrpWarning>,

    /// 計算耗時（毫秒）
    pub calculation_time_ms: Option<u128>,
}

impl MrpResult {
    /// 創建空的計算結果
    pub fn empty() -> Self {
        Self {
            planned_orders: Vec::new(),
            pegging: std::collections::HashMap::new(),
            warnings: Vec::new(),
            calculation_time_ms: None,
        }
    }

    /// 添加警告
    pub fn add_warning(&mut self, warning: MrpWarning) {
        self.warnings.push(warning);
    }
}

/// MRP 警告
#[derive(Debug, Clone)]
pub struct MrpWarning {
    pub component_id: String,
    pub message: String,
    pub severity: WarningSeverity,
}

impl MrpWarning {
    pub fn new(component_id: String, message: String, severity: WarningSeverity) -> Self {
        Self {
            component_id,
            message,
            severity,
        }
    }

    pub fn info(component_id: String, message: String) -> Self {
        Self::new(component_id, message, WarningSeverity::Info)
    }

    pub fn warning(component_id: String, message: String) -> Self {
        Self::new(component_id, message, WarningSeverity::Warning)
    }

    pub fn error(component_id: String, message: String) -> Self {
        Self::new(component_id, message, WarningSeverity::Error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// 單物料 MRP 計算結果
#[derive(Debug, Clone)]
pub struct ComponentMrpResult {
    pub component_id: String,
    pub planned_orders: Vec<mrp_core::PlannedOrder>,
}
