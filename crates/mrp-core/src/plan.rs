//! 計劃訂單模型

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 計劃訂單類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlannedOrderType {
    /// 採購
    Purchase,
    /// 生產
    Production,
    /// 調撥
    Transfer,
}

/// 計劃訂單（MRP計算結果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedOrder {
    /// 計劃訂單ID
    pub id: Uuid,

    /// 物料ID
    pub component_id: String,

    /// 計劃數量
    pub quantity: Decimal,

    /// 需求日期（完成日期）
    pub required_date: NaiveDate,

    /// 下單日期（開始日期）
    pub order_date: NaiveDate,

    /// 訂單類型
    pub order_type: PlannedOrderType,

    /// 供應商/工作中心
    pub source_id: Option<String>,

    /// 需求來源追溯
    pub pegging: Vec<PeggingRecord>,
}

impl PlannedOrder {
    /// 創建新的計劃訂單
    pub fn new(
        component_id: String,
        quantity: Decimal,
        required_date: NaiveDate,
        order_date: NaiveDate,
        order_type: PlannedOrderType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_id,
            quantity,
            required_date,
            order_date,
            order_type,
            source_id: None,
            pegging: Vec::new(),
        }
    }

    /// 建構器模式：設置供應商/工作中心
    pub fn with_source_id(mut self, source_id: String) -> Self {
        self.source_id = Some(source_id);
        self
    }

    /// 建構器模式：設置需求追溯
    pub fn with_pegging(mut self, pegging: Vec<PeggingRecord>) -> Self {
        self.pegging = pegging;
        self
    }

    /// 添加追溯記錄
    pub fn add_pegging(&mut self, record: PeggingRecord) {
        self.pegging.push(record);
    }

    /// 計算提前期（天數）
    pub fn lead_time_days(&self) -> i64 {
        (self.required_date - self.order_date).num_days()
    }

    /// 檢查是否為採購訂單
    pub fn is_purchase(&self) -> bool {
        self.order_type == PlannedOrderType::Purchase
    }

    /// 檢查是否為生產訂單
    pub fn is_production(&self) -> bool {
        self.order_type == PlannedOrderType::Production
    }
}

/// 需求追溯記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeggingRecord {
    /// 源需求ID
    pub demand_id: Uuid,

    /// 追溯數量
    pub quantity: Decimal,

    /// 追溯路徑（多級）
    pub path: Vec<String>,
}

impl PeggingRecord {
    /// 創建新的追溯記錄
    pub fn new(demand_id: Uuid, quantity: Decimal) -> Self {
        Self {
            demand_id,
            quantity,
            path: Vec::new(),
        }
    }

    /// 建構器模式：設置追溯路徑
    pub fn with_path(mut self, path: Vec<String>) -> Self {
        self.path = path;
        self
    }

    /// 添加路徑節點
    pub fn add_path_node(&mut self, node: String) {
        self.path.push(node);
    }

    /// 獲取追溯深度（層級）
    pub fn depth(&self) -> usize {
        self.path.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_planned_order() {
        let order = PlannedOrder::new(
            "BIKE-001".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(),
            PlannedOrderType::Production,
        );

        assert_eq!(order.component_id, "BIKE-001");
        assert_eq!(order.quantity, Decimal::from(100));
        assert_eq!(order.lead_time_days(), 7);
        assert!(order.is_production());
        assert!(!order.is_purchase());
    }

    #[test]
    fn test_planned_order_builder() {
        let pegging = vec![PeggingRecord::new(Uuid::new_v4(), Decimal::from(50))];

        let order = PlannedOrder::new(
            "FRAME-001".to_string(),
            Decimal::from(50),
            NaiveDate::from_ymd_opt(2025, 10, 28).unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(),
            PlannedOrderType::Purchase,
        )
        .with_source_id("VENDOR-01".to_string())
        .with_pegging(pegging);

        assert_eq!(order.source_id, Some("VENDOR-01".to_string()));
        assert_eq!(order.pegging.len(), 1);
        assert!(order.is_purchase());
    }

    #[test]
    fn test_pegging_record() {
        let mut record = PeggingRecord::new(Uuid::new_v4(), Decimal::from(100))
            .with_path(vec!["BIKE-001".to_string(), "FRAME-001".to_string()]);

        assert_eq!(record.depth(), 2);

        record.add_path_node("WHEEL-001".to_string());
        assert_eq!(record.depth(), 3);
    }
}
