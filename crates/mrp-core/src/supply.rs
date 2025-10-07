//! 供應模型

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 供應類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplyType {
    /// 現有庫存
    OnHand,
    /// 採購訂單
    PurchaseOrder,
    /// 生產工單
    WorkOrder,
    /// 調撥在途
    Transfer,
    /// 計劃訂單（MRP生成）
    PlannedOrder,
}

/// 供應
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supply {
    /// 供應ID
    pub id: Uuid,

    /// 物料ID
    pub component_id: String,

    /// 供應數量
    pub quantity: Decimal,

    /// 可用日期
    pub available_date: NaiveDate,

    /// 供應類型
    pub supply_type: SupplyType,

    /// 來源單據
    pub source_ref: Option<String>,

    /// 是否已確認（確認的訂單不會被 MRP 修改）
    pub is_firm: bool,
}

impl Supply {
    /// 創建新的供應
    pub fn new(
        component_id: String,
        quantity: Decimal,
        available_date: NaiveDate,
        supply_type: SupplyType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_id,
            quantity,
            available_date,
            supply_type,
            source_ref: None,
            is_firm: false,
        }
    }

    /// 建構器模式：設置來源單據
    pub fn with_source_ref(mut self, source_ref: String) -> Self {
        self.source_ref = Some(source_ref);
        self
    }

    /// 建構器模式：設置為確認狀態
    pub fn as_firm(mut self) -> Self {
        self.is_firm = true;
        self
    }

    /// 檢查是否為計劃供應（MRP 生成）
    pub fn is_planned(&self) -> bool {
        self.supply_type == SupplyType::PlannedOrder
    }

    /// 檢查是否可被 MRP 調整
    pub fn is_adjustable(&self) -> bool {
        !self.is_firm && self.is_planned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_supply() {
        let supply = Supply::new(
            "BIKE-001".to_string(),
            Decimal::from(50),
            NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(),
            SupplyType::PurchaseOrder,
        );

        assert_eq!(supply.component_id, "BIKE-001");
        assert_eq!(supply.quantity, Decimal::from(50));
        assert!(!supply.is_firm);
        assert!(!supply.is_planned());
    }

    #[test]
    fn test_supply_builder() {
        let supply = Supply::new(
            "FRAME-001".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(),
            SupplyType::PlannedOrder,
        )
        .with_source_ref("PO-12345".to_string())
        .as_firm();

        assert_eq!(supply.source_ref, Some("PO-12345".to_string()));
        assert!(supply.is_firm);
        assert!(supply.is_planned());
        assert!(!supply.is_adjustable());
    }
}
