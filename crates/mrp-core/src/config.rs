//! MRP 配置模型

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 物料MRP參數配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrpConfig {
    /// 物料ID
    pub component_id: String,

    /// 提前期（天）
    pub lead_time_days: u32,

    /// 批量規則
    pub lot_sizing_rule: LotSizingRule,

    /// 固定批量（如果適用）
    pub fixed_lot_size: Option<Decimal>,

    /// 最小訂購量
    pub minimum_order_qty: Option<Decimal>,

    /// 最大訂購量
    pub maximum_order_qty: Option<Decimal>,

    /// 訂購倍數（必須是此倍數）
    pub order_multiple: Option<Decimal>,

    /// 安全庫存
    pub safety_stock: Decimal,

    /// 計劃時界（天）
    pub planning_horizon_days: u32,

    /// 採購/生產標記
    pub procurement_type: ProcurementType,

    /// 是否啟用 MRP（有些物料可能不需要 MRP）
    pub mrp_enabled: bool,

    /// 是否允許負庫存
    /// - true: 允許預計庫存為負值（適用於可超賣或 MTO 模式）
    /// - false: 不允許負庫存，當庫存不足時立即觸發計劃訂單（預設）
    ///
    /// 使用場景：
    /// - 允許：按單生產(MTO)、服務類物料、虛擬件
    /// - 不允許：實體庫存管理、批量生產(MTS)
    pub allow_negative_inventory: bool,
}

impl MrpConfig {
    /// 創建新的 MRP 配置
    pub fn new(component_id: String, lead_time_days: u32, procurement_type: ProcurementType) -> Self {
        Self {
            component_id,
            lead_time_days,
            lot_sizing_rule: LotSizingRule::LotForLot,
            fixed_lot_size: None,
            minimum_order_qty: None,
            maximum_order_qty: None,
            order_multiple: None,
            safety_stock: Decimal::ZERO,
            planning_horizon_days: 90,
            procurement_type,
            mrp_enabled: true,
            allow_negative_inventory: false, // 預設不允許負庫存（保守策略）
        }
    }

    /// 建構器模式：設置批量規則
    pub fn with_lot_sizing_rule(mut self, rule: LotSizingRule) -> Self {
        self.lot_sizing_rule = rule;
        self
    }

    /// 建構器模式：設置固定批量
    pub fn with_fixed_lot_size(mut self, size: Decimal) -> Self {
        self.fixed_lot_size = Some(size);
        self
    }

    /// 建構器模式：設置最小訂購量
    pub fn with_minimum_order_qty(mut self, qty: Decimal) -> Self {
        self.minimum_order_qty = Some(qty);
        self
    }

    /// 建構器模式：設置最大訂購量
    pub fn with_maximum_order_qty(mut self, qty: Decimal) -> Self {
        self.maximum_order_qty = Some(qty);
        self
    }

    /// 建構器模式：設置訂購倍數
    pub fn with_order_multiple(mut self, multiple: Decimal) -> Self {
        self.order_multiple = Some(multiple);
        self
    }

    /// 建構器模式：設置安全庫存
    pub fn with_safety_stock(mut self, stock: Decimal) -> Self {
        self.safety_stock = stock;
        self
    }

    /// 建構器模式：設置計劃時界
    pub fn with_planning_horizon(mut self, days: u32) -> Self {
        self.planning_horizon_days = days;
        self
    }

    /// 建構器模式：設置是否允許負庫存
    ///
    /// # 參數
    /// * `allow` - true 允許負庫存，false 不允許（預設）
    ///
    /// # 範例
    /// ```
    /// # use mrp_core::{MrpConfig, ProcurementType};
    /// let config = MrpConfig::new("PART-001".to_string(), 3, ProcurementType::Buy)
    ///     .with_allow_negative_inventory(true); // MTO 模式，允許負庫存
    /// ```
    pub fn with_allow_negative_inventory(mut self, allow: bool) -> Self {
        self.allow_negative_inventory = allow;
        self
    }

    /// 調整訂購量以符合批量規則
    pub fn adjust_order_quantity(&self, mut quantity: Decimal) -> Decimal {
        // 應用最小訂購量
        if let Some(min_qty) = self.minimum_order_qty {
            if quantity < min_qty {
                quantity = min_qty;
            }
        }

        // 應用訂購倍數
        if let Some(multiple) = self.order_multiple {
            if multiple > Decimal::ZERO {
                let remainder = quantity % multiple;
                if remainder > Decimal::ZERO {
                    quantity = quantity - remainder + multiple;
                }
            }
        }

        // 應用最大訂購量
        if let Some(max_qty) = self.maximum_order_qty {
            if quantity > max_qty {
                quantity = max_qty;
            }
        }

        quantity
    }

    /// 檢查是否需要 MRP 計算
    pub fn needs_mrp(&self) -> bool {
        self.mrp_enabled
    }
}

/// 採購類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcurementType {
    /// 採購
    Buy,
    /// 生產
    Make,
    /// 調撥
    Transfer,
}

/// 批量規則
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LotSizingRule {
    /// 批對批（Lot for Lot）- 按實際需求訂購
    LotForLot,

    /// 固定訂購量（Fixed Order Quantity）- 每次固定數量
    FixedOrderQuantity,

    /// 經濟訂購量（Economic Order Quantity）
    EconomicOrderQuantity,

    /// 週期訂購量（Period Order Quantity）- 合併週期內需求
    PeriodOrderQuantity,

    /// 最小-最大（Min-Max）
    MinMax,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_config() {
        let config = MrpConfig::new(
            "BIKE-001".to_string(),
            5,
            ProcurementType::Make,
        );

        assert_eq!(config.component_id, "BIKE-001");
        assert_eq!(config.lead_time_days, 5);
        assert_eq!(config.lot_sizing_rule, LotSizingRule::LotForLot);
        assert!(config.needs_mrp());
    }

    #[test]
    fn test_config_builder() {
        let config = MrpConfig::new(
            "FRAME-001".to_string(),
            7,
            ProcurementType::Buy,
        )
        .with_lot_sizing_rule(LotSizingRule::FixedOrderQuantity)
        .with_fixed_lot_size(Decimal::from(100))
        .with_minimum_order_qty(Decimal::from(50))
        .with_safety_stock(Decimal::from(20));

        assert_eq!(config.lot_sizing_rule, LotSizingRule::FixedOrderQuantity);
        assert_eq!(config.fixed_lot_size, Some(Decimal::from(100)));
        assert_eq!(config.minimum_order_qty, Some(Decimal::from(50)));
        assert_eq!(config.safety_stock, Decimal::from(20));
    }

    #[test]
    fn test_adjust_order_quantity() {
        let config = MrpConfig::new(
            "WHEEL-001".to_string(),
            3,
            ProcurementType::Buy,
        )
        .with_minimum_order_qty(Decimal::from(50))
        .with_maximum_order_qty(Decimal::from(500))
        .with_order_multiple(Decimal::from(10));

        // 低於最小訂購量
        assert_eq!(config.adjust_order_quantity(Decimal::from(30)), Decimal::from(50));

        // 需要調整到倍數
        assert_eq!(config.adjust_order_quantity(Decimal::from(75)), Decimal::from(80));

        // 超過最大訂購量
        assert_eq!(config.adjust_order_quantity(Decimal::from(600)), Decimal::from(500));
    }

    #[test]
    fn test_order_multiple_adjustment() {
        let config = MrpConfig::new(
            "SCREW-001".to_string(),
            1,
            ProcurementType::Buy,
        )
        .with_order_multiple(Decimal::from(100));

        // 123 應該調整為 200
        assert_eq!(config.adjust_order_quantity(Decimal::from(123)), Decimal::from(200));

        // 200 已經是倍數，不需調整
        assert_eq!(config.adjust_order_quantity(Decimal::from(200)), Decimal::from(200));
    }
}
