//! 需求模型

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 需求類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemandType {
    /// 銷售訂單
    SalesOrder,
    /// 銷售預測
    Forecast,
    /// 安全庫存
    SafetyStock,
    /// 相依需求（BOM展開）
    Dependent,
}

/// 需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demand {
    /// 需求ID
    pub id: Uuid,

    /// 物料ID
    pub component_id: String,

    /// 需求數量
    pub quantity: Decimal,

    /// 需求日期
    pub required_date: NaiveDate,

    /// 需求類型
    pub demand_type: DemandType,

    /// 來源單據（如銷售訂單號）
    pub source_ref: Option<String>,

    /// 優先級（1-10，10最高）
    pub priority: u8,

    /// 工廠/組織
    pub plant_id: Option<String>,
}

impl Demand {
    /// 創建新的需求
    pub fn new(
        component_id: String,
        quantity: Decimal,
        required_date: NaiveDate,
        demand_type: DemandType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_id,
            quantity,
            required_date,
            demand_type,
            source_ref: None,
            priority: 5,
            plant_id: None,
        }
    }

    /// 建構器模式：設置來源單據
    pub fn with_source_ref(mut self, source_ref: String) -> Self {
        self.source_ref = Some(source_ref);
        self
    }

    /// 建構器模式：設置優先級
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    /// 建構器模式：設置工廠
    pub fn with_plant_id(mut self, plant_id: String) -> Self {
        self.plant_id = Some(plant_id);
        self
    }

    /// 檢查是否為獨立需求
    pub fn is_independent(&self) -> bool {
        matches!(
            self.demand_type,
            DemandType::SalesOrder | DemandType::Forecast | DemandType::SafetyStock
        )
    }

    /// 檢查是否為相依需求
    pub fn is_dependent(&self) -> bool {
        self.demand_type == DemandType::Dependent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_demand() {
        let demand = Demand::new(
            "BIKE-001".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            DemandType::SalesOrder,
        );

        assert_eq!(demand.component_id, "BIKE-001");
        assert_eq!(demand.quantity, Decimal::from(100));
        assert_eq!(demand.priority, 5);
        assert!(demand.is_independent());
    }

    #[test]
    fn test_demand_builder() {
        let demand = Demand::new(
            "FRAME-001".to_string(),
            Decimal::from(50),
            NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
            DemandType::Dependent,
        )
        .with_source_ref("SO-12345".to_string())
        .with_priority(8)
        .with_plant_id("PLANT-01".to_string());

        assert_eq!(demand.source_ref, Some("SO-12345".to_string()));
        assert_eq!(demand.priority, 8);
        assert_eq!(demand.plant_id, Some("PLANT-01".to_string()));
        assert!(demand.is_dependent());
    }
}
