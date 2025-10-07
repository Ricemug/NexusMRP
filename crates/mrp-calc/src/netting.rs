//! 淨需求計算

use chrono::NaiveDate;
use rust_decimal::Decimal;

/// 淨需求計算結果
#[derive(Debug, Clone)]
pub struct NetRequirement {
    /// 日期
    pub date: NaiveDate,
    /// 總需求
    pub gross_requirement: Decimal,
    /// 預計收貨
    pub scheduled_receipt: Decimal,
    /// 預計庫存
    pub projected_on_hand: Decimal,
    /// 淨需求
    pub net_requirement: Decimal,
}

impl NetRequirement {
    /// 創建新的淨需求記錄
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            gross_requirement: Decimal::ZERO,
            scheduled_receipt: Decimal::ZERO,
            projected_on_hand: Decimal::ZERO,
            net_requirement: Decimal::ZERO,
        }
    }
}

/// 淨需求計算器
pub struct NettingCalculator;

impl NettingCalculator {
    /// 計算淨需求
    ///
    /// # 參數
    /// * `allow_negative_inventory` - 是否允許負庫存
    ///   - false: 不允許庫存低於安全庫存，當 projected_on_hand < safety_stock 時產生淨需求
    ///   - true: 允許負庫存，只有當 projected_on_hand < 0 時才產生淨需求（忽略安全庫存）
    pub fn calculate(
        demands: &[mrp_core::Demand],
        supplies: &[mrp_core::Supply],
        initial_inventory: Decimal,
        safety_stock: Decimal,
        time_buckets: &[NaiveDate],
        allow_negative_inventory: bool,
    ) -> mrp_core::Result<Vec<NetRequirement>> {
        let mut results = Vec::new();
        let mut current_inventory = initial_inventory;

        for &date in time_buckets {
            // 該日期的總需求
            let gross_req = demands
                .iter()
                .filter(|d| d.required_date == date)
                .map(|d| d.quantity)
                .sum::<Decimal>();

            // 該日期的預計收貨
            let scheduled_receipt = supplies
                .iter()
                .filter(|s| s.available_date == date)
                .map(|s| s.quantity)
                .sum::<Decimal>();

            // 計算預計庫存
            let projected_on_hand = current_inventory + scheduled_receipt - gross_req;

            // 計算淨需求
            let net_req = if allow_negative_inventory {
                // 允許負庫存：只有當預計庫存為負時才產生淨需求
                if projected_on_hand < Decimal::ZERO {
                    -projected_on_hand // 補足到 0
                } else {
                    Decimal::ZERO
                }
            } else {
                // 不允許負庫存：低於安全庫存時就要產生淨需求
                if projected_on_hand < safety_stock {
                    safety_stock - projected_on_hand
                } else {
                    Decimal::ZERO
                }
            };

            results.push(NetRequirement {
                date,
                gross_requirement: gross_req,
                scheduled_receipt,
                projected_on_hand,
                net_requirement: net_req,
            });

            current_inventory = projected_on_hand;
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mrp_core::{Demand, DemandType, Supply, SupplyType};
    use rust_decimal::Decimal;

    #[test]
    fn test_netting_calculation_simple() {
        let time_buckets = vec![
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
            NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(),
        ];

        let demands = vec![
            Demand::new(
                "TEST-001".to_string(),
                Decimal::from(100),
                NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                DemandType::SalesOrder,
            ),
            Demand::new(
                "TEST-001".to_string(),
                Decimal::from(50),
                NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
                DemandType::SalesOrder,
            ),
        ];

        let supplies = vec![
            Supply::new(
                "TEST-001".to_string(),
                Decimal::from(30),
                NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
                SupplyType::PurchaseOrder,
            ),
        ];

        let initial_inventory = Decimal::from(20);
        let safety_stock = Decimal::from(10);

        let result = NettingCalculator::calculate(
            &demands,
            &supplies,
            initial_inventory,
            safety_stock,
            &time_buckets,
            false, // 不允許負庫存
        ).unwrap();

        assert_eq!(result.len(), 3);

        // 11/1: 庫存20 - 需求100 = -80，低於安全庫存10，淨需求 = 90
        assert_eq!(result[0].gross_requirement, Decimal::from(100));
        assert_eq!(result[0].scheduled_receipt, Decimal::ZERO);
        assert_eq!(result[0].net_requirement, Decimal::from(90));

        // 11/5: 有供應30，需求50
        assert_eq!(result[1].gross_requirement, Decimal::from(50));
        assert_eq!(result[1].scheduled_receipt, Decimal::from(30));
    }

    #[test]
    fn test_netting_with_sufficient_inventory() {
        let time_buckets = vec![
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
        ];

        let demands = vec![
            Demand::new(
                "TEST-002".to_string(),
                Decimal::from(50),
                NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                DemandType::SalesOrder,
            ),
        ];

        let supplies = vec![];
        let initial_inventory = Decimal::from(100);
        let safety_stock = Decimal::from(20);

        let result = NettingCalculator::calculate(
            &demands,
            &supplies,
            initial_inventory,
            safety_stock,
            &time_buckets,
            false, // 不允許負庫存
        ).unwrap();

        assert_eq!(result.len(), 1);
        // 庫存100 - 需求50 = 50，高於安全庫存20，無淨需求
        assert_eq!(result[0].net_requirement, Decimal::ZERO);
        assert_eq!(result[0].projected_on_hand, Decimal::from(50));
    }

    #[test]
    fn test_netting_cumulative_effect() {
        let time_buckets = vec![
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2025, 11, 3).unwrap(),
        ];

        let demands = vec![
            Demand::new(
                "TEST-003".to_string(),
                Decimal::from(30),
                NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                DemandType::SalesOrder,
            ),
            Demand::new(
                "TEST-003".to_string(),
                Decimal::from(30),
                NaiveDate::from_ymd_opt(2025, 11, 2).unwrap(),
                DemandType::SalesOrder,
            ),
            Demand::new(
                "TEST-003".to_string(),
                Decimal::from(30),
                NaiveDate::from_ymd_opt(2025, 11, 3).unwrap(),
                DemandType::SalesOrder,
            ),
        ];

        let supplies = vec![];
        let initial_inventory = Decimal::from(100);
        let safety_stock = Decimal::ZERO;

        let result = NettingCalculator::calculate(
            &demands,
            &supplies,
            initial_inventory,
            safety_stock,
            &time_buckets,
            false, // 不允許負庫存
        ).unwrap();

        // 驗證庫存遞減
        assert_eq!(result[0].projected_on_hand, Decimal::from(70)); // 100 - 30
        assert_eq!(result[1].projected_on_hand, Decimal::from(40)); // 70 - 30
        assert_eq!(result[2].projected_on_hand, Decimal::from(10)); // 40 - 30
    }

    #[test]
    fn test_allow_negative_inventory_false() {
        // 測試不允許負庫存的情況
        let time_buckets = vec![NaiveDate::from_ymd_opt(2025, 11, 1).unwrap()];

        let demands = vec![Demand::new(
            "TEST-NEG-1".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            DemandType::SalesOrder,
        )];

        let supplies = vec![];
        let initial_inventory = Decimal::from(30); // 庫存不足
        let safety_stock = Decimal::from(10);

        let result = NettingCalculator::calculate(
            &demands,
            &supplies,
            initial_inventory,
            safety_stock,
            &time_buckets,
            false, // 不允許負庫存
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        // 預計庫存: 30 - 100 = -70
        assert_eq!(result[0].projected_on_hand, Decimal::from(-70));
        // 不允許負庫存：需補足到安全庫存，淨需求 = 10 - (-70) = 80
        assert_eq!(result[0].net_requirement, Decimal::from(80));
    }

    #[test]
    fn test_allow_negative_inventory_true() {
        // 測試允許負庫存的情況
        let time_buckets = vec![NaiveDate::from_ymd_opt(2025, 11, 1).unwrap()];

        let demands = vec![Demand::new(
            "TEST-NEG-2".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            DemandType::SalesOrder,
        )];

        let supplies = vec![];
        let initial_inventory = Decimal::from(30); // 庫存不足
        let safety_stock = Decimal::from(10); // 安全庫存被忽略

        let result = NettingCalculator::calculate(
            &demands,
            &supplies,
            initial_inventory,
            safety_stock,
            &time_buckets,
            true, // 允許負庫存
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        // 預計庫存: 30 - 100 = -70
        assert_eq!(result[0].projected_on_hand, Decimal::from(-70));
        // 允許負庫存：只需補足到 0，淨需求 = 70
        assert_eq!(result[0].net_requirement, Decimal::from(70));
    }

    #[test]
    fn test_allow_negative_inventory_with_positive_stock() {
        // 測試允許負庫存但庫存仍為正的情況
        let time_buckets = vec![NaiveDate::from_ymd_opt(2025, 11, 1).unwrap()];

        let demands = vec![Demand::new(
            "TEST-NEG-3".to_string(),
            Decimal::from(50),
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            DemandType::SalesOrder,
        )];

        let supplies = vec![];
        let initial_inventory = Decimal::from(100); // 庫存充足
        let safety_stock = Decimal::from(30);

        let result = NettingCalculator::calculate(
            &demands,
            &supplies,
            initial_inventory,
            safety_stock,
            &time_buckets,
            true, // 允許負庫存
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        // 預計庫存: 100 - 50 = 50
        assert_eq!(result[0].projected_on_hand, Decimal::from(50));
        // 允許負庫存模式：庫存為正，不產生淨需求（即使低於安全庫存）
        assert_eq!(result[0].net_requirement, Decimal::ZERO);
    }
}
