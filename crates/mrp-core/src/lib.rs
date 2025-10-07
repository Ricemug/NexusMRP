//! # MRP Core
//!
//! 核心資料模型與類型定義

pub mod calendar;
pub mod config;
pub mod demand;
pub mod inventory;
pub mod plan;
pub mod supply;

// Re-export 主要類型
pub use calendar::{ShiftSchedule, WorkCalendar};
pub use config::{LotSizingRule, MrpConfig, ProcurementType};
pub use demand::{Demand, DemandType};
pub use inventory::Inventory;
pub use plan::{PeggingRecord, PlannedOrder, PlannedOrderType};
pub use supply::{Supply, SupplyType};

/// MRP 錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum MrpError {
    #[error("找不到物料配置: {0}")]
    ConfigNotFound(String),

    #[error("BOM 展開錯誤: {0}")]
    BomExplosionError(String),

    #[error("拓撲排序錯誤: {0}")]
    TopologicalSortError(String),

    #[error("批量規則缺少必要參數")]
    MissingLotSize,

    #[error("無效的日期: {0}")]
    InvalidDate(String),

    #[error("計算錯誤: {0}")]
    CalculationError(String),

    #[error("其他錯誤: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MrpError>;
