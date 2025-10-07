//! 庫存模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 庫存狀態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// 物料ID
    pub component_id: String,

    /// 現有庫存
    pub on_hand_qty: Decimal,

    /// 安全庫存
    pub safety_stock: Decimal,

    /// 已分配數量（鎖定）
    pub allocated_qty: Decimal,

    /// 可用庫存（現有 - 已分配）
    pub available_qty: Decimal,

    /// 倉庫
    pub warehouse_id: Option<String>,
}

impl Inventory {
    /// 創建新的庫存記錄
    pub fn new(component_id: String, on_hand_qty: Decimal, safety_stock: Decimal) -> Self {
        let available_qty = on_hand_qty;
        Self {
            component_id,
            on_hand_qty,
            safety_stock,
            allocated_qty: Decimal::ZERO,
            available_qty,
            warehouse_id: None,
        }
    }

    /// 建構器模式：設置已分配數量
    pub fn with_allocated_qty(mut self, allocated_qty: Decimal) -> Self {
        self.allocated_qty = allocated_qty;
        self.available_qty = self.on_hand_qty - allocated_qty;
        self
    }

    /// 建構器模式：設置倉庫
    pub fn with_warehouse_id(mut self, warehouse_id: String) -> Self {
        self.warehouse_id = Some(warehouse_id);
        self
    }

    /// 計算可用庫存
    pub fn calculate_available(&mut self) {
        self.available_qty = self.on_hand_qty - self.allocated_qty;
    }

    /// 檢查庫存是否低於安全庫存
    pub fn is_below_safety_stock(&self) -> bool {
        self.available_qty < self.safety_stock
    }

    /// 獲取需要補充的數量
    pub fn replenishment_needed(&self) -> Decimal {
        if self.is_below_safety_stock() {
            self.safety_stock - self.available_qty
        } else {
            Decimal::ZERO
        }
    }

    /// 分配庫存
    pub fn allocate(&mut self, quantity: Decimal) -> Result<(), String> {
        if quantity > self.available_qty {
            return Err(format!(
                "庫存不足：需要 {}, 可用 {}",
                quantity, self.available_qty
            ));
        }
        self.allocated_qty += quantity;
        self.calculate_available();
        Ok(())
    }

    /// 釋放已分配的庫存
    pub fn deallocate(&mut self, quantity: Decimal) -> Result<(), String> {
        if quantity > self.allocated_qty {
            return Err(format!(
                "釋放數量超過已分配數量：釋放 {}, 已分配 {}",
                quantity, self.allocated_qty
            ));
        }
        self.allocated_qty -= quantity;
        self.calculate_available();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_inventory() {
        let inventory = Inventory::new(
            "BIKE-001".to_string(),
            Decimal::from(100),
            Decimal::from(20),
        );

        assert_eq!(inventory.component_id, "BIKE-001");
        assert_eq!(inventory.on_hand_qty, Decimal::from(100));
        assert_eq!(inventory.safety_stock, Decimal::from(20));
        assert_eq!(inventory.available_qty, Decimal::from(100));
        assert!(!inventory.is_below_safety_stock());
    }

    #[test]
    fn test_inventory_allocation() {
        let mut inventory = Inventory::new(
            "FRAME-001".to_string(),
            Decimal::from(100),
            Decimal::from(10),
        );

        // 分配庫存
        assert!(inventory.allocate(Decimal::from(50)).is_ok());
        assert_eq!(inventory.allocated_qty, Decimal::from(50));
        assert_eq!(inventory.available_qty, Decimal::from(50));

        // 超量分配應該失敗
        assert!(inventory.allocate(Decimal::from(60)).is_err());

        // 釋放庫存
        assert!(inventory.deallocate(Decimal::from(30)).is_ok());
        assert_eq!(inventory.allocated_qty, Decimal::from(20));
        assert_eq!(inventory.available_qty, Decimal::from(80));
    }

    #[test]
    fn test_replenishment_needed() {
        let inventory = Inventory::new(
            "WHEEL-001".to_string(),
            Decimal::from(5),
            Decimal::from(20),
        );

        assert!(inventory.is_below_safety_stock());
        assert_eq!(inventory.replenishment_needed(), Decimal::from(15));
    }
}
