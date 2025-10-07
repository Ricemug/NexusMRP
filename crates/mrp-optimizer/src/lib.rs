//! # MRP Optimizer
//!
//! 優化算法模組（產能、排程、約束求解）

pub mod capacity;
pub mod constraint;
pub mod scheduling;

// Re-export 主要類型
pub use capacity::CapacityPlanner;
pub use scheduling::Scheduler;

/// 優化結果
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// 是否可行
    pub feasible: bool,

    /// 優化後的計劃訂單
    pub optimized_orders: Vec<mrp_core::PlannedOrder>,

    /// 優化信息
    pub messages: Vec<String>,
}

impl OptimizationResult {
    /// 創建可行的優化結果
    pub fn feasible(optimized_orders: Vec<mrp_core::PlannedOrder>) -> Self {
        Self {
            feasible: true,
            optimized_orders,
            messages: Vec::new(),
        }
    }

    /// 創建不可行的優化結果
    pub fn infeasible(message: String) -> Self {
        Self {
            feasible: false,
            optimized_orders: Vec::new(),
            messages: vec![message],
        }
    }
}
