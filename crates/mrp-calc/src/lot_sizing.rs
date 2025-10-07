//! 批量規則實現

use mrp_core::{LotSizingRule, MrpConfig, PlannedOrder, PlannedOrderType, ProcurementType};
use rust_decimal::Decimal;

use crate::netting::NetRequirement;

/// 批量規則計算器
pub struct LotSizingCalculator;

impl LotSizingCalculator {
    /// 應用批量規則
    pub fn apply(
        component_id: &str,
        net_requirements: &[NetRequirement],
        config: &MrpConfig,
        calendar: &mrp_core::WorkCalendar,
    ) -> mrp_core::Result<Vec<PlannedOrder>> {
        match config.lot_sizing_rule {
            LotSizingRule::LotForLot => {
                Self::lot_for_lot(component_id, net_requirements, config, calendar)
            }
            LotSizingRule::FixedOrderQuantity => {
                Self::fixed_order_quantity(component_id, net_requirements, config, calendar)
            }
            LotSizingRule::EconomicOrderQuantity => {
                Self::economic_order_quantity(component_id, net_requirements, config, calendar)
            }
            LotSizingRule::PeriodOrderQuantity => {
                Self::period_order_quantity(component_id, net_requirements, config, calendar)
            }
            LotSizingRule::MinMax => {
                Self::min_max(component_id, net_requirements, config, calendar)
            }
        }
    }

    /// 批對批（Lot for Lot）
    fn lot_for_lot(
        component_id: &str,
        net_requirements: &[NetRequirement],
        config: &MrpConfig,
        calendar: &mrp_core::WorkCalendar,
    ) -> mrp_core::Result<Vec<PlannedOrder>> {
        let mut planned_orders = Vec::new();

        for req in net_requirements {
            if req.net_requirement > Decimal::ZERO {
                let order_date =
                    calendar.subtract_working_days(req.date, config.lead_time_days);

                let quantity = config.adjust_order_quantity(req.net_requirement);

                planned_orders.push(PlannedOrder::new(
                    component_id.to_string(),
                    quantity,
                    req.date,
                    order_date,
                    Self::determine_order_type(config.procurement_type),
                ));
            }
        }

        Ok(planned_orders)
    }

    /// 固定訂購量（Fixed Order Quantity）
    /// 每次訂購固定數量，可能需要多次訂購來滿足需求
    fn fixed_order_quantity(
        component_id: &str,
        net_requirements: &[NetRequirement],
        config: &MrpConfig,
        calendar: &mrp_core::WorkCalendar,
    ) -> mrp_core::Result<Vec<PlannedOrder>> {
        let fixed_lot_size = config
            .fixed_lot_size
            .ok_or_else(|| mrp_core::MrpError::MissingLotSize)?;

        let mut planned_orders = Vec::new();
        let mut remaining_inventory = Decimal::ZERO;

        for req in net_requirements {
            // 計算可用庫存（包含前期剩餘）
            remaining_inventory -= req.gross_requirement;
            remaining_inventory += req.scheduled_receipt;

            // 如果低於安全庫存，需要下單
            if remaining_inventory < config.safety_stock {
                let shortage = config.safety_stock - remaining_inventory;

                // 計算需要幾批固定批量
                let batches_needed = {
                    let ratio = shortage / fixed_lot_size;
                    ratio.ceil().to_string().parse::<u32>().unwrap_or(1)
                };

                let order_quantity = fixed_lot_size * Decimal::from(batches_needed);
                let adjusted_quantity = config.adjust_order_quantity(order_quantity);

                let order_date =
                    calendar.subtract_working_days(req.date, config.lead_time_days);

                planned_orders.push(PlannedOrder::new(
                    component_id.to_string(),
                    adjusted_quantity,
                    req.date,
                    order_date,
                    Self::determine_order_type(config.procurement_type),
                ));

                remaining_inventory += adjusted_quantity;
            }
        }

        Ok(planned_orders)
    }

    /// 經濟訂購量（EOQ）
    /// 基於成本優化的批量計算
    /// EOQ = sqrt(2 * 年需求量 * 訂購成本 / 持有成本)
    fn economic_order_quantity(
        component_id: &str,
        net_requirements: &[NetRequirement],
        config: &MrpConfig,
        calendar: &mrp_core::WorkCalendar,
    ) -> mrp_core::Result<Vec<PlannedOrder>> {
        // 簡化實現：如果沒有配置 EOQ 參數，退回到固定批量或 LFL
        let eoq_size = if let Some(fixed_size) = config.fixed_lot_size {
            fixed_size
        } else {
            // 如果沒有配置，計算簡化的 EOQ
            // 預設使用年需求量的平方根作為批量
            let total_requirement: Decimal = net_requirements
                .iter()
                .map(|r| r.net_requirement)
                .sum();

            if total_requirement > Decimal::ZERO {
                // 簡化公式：sqrt(total_demand) * 10
                // 使用 f64 計算平方根，然後轉回 Decimal
                let total_f64 = total_requirement.to_string().parse::<f64>().unwrap_or(100.0);
                let sqrt_value = total_f64.sqrt();
                Decimal::try_from(sqrt_value * 10.0).unwrap_or(Decimal::from(100))
            } else {
                Decimal::from(100) // 預設批量
            }
        };

        // 使用計算出的 EOQ 作為固定批量
        let mut planned_orders = Vec::new();
        let mut remaining_inventory = Decimal::ZERO;

        for req in net_requirements {
            remaining_inventory -= req.gross_requirement;
            remaining_inventory += req.scheduled_receipt;

            if remaining_inventory < config.safety_stock {
                let shortage = config.safety_stock - remaining_inventory;
                let batches_needed = {
                    let ratio = shortage / eoq_size;
                    ratio.ceil().to_string().parse::<u32>().unwrap_or(1)
                };
                let order_quantity = eoq_size * Decimal::from(batches_needed);
                let adjusted_quantity = config.adjust_order_quantity(order_quantity);

                let order_date =
                    calendar.subtract_working_days(req.date, config.lead_time_days);

                planned_orders.push(PlannedOrder::new(
                    component_id.to_string(),
                    adjusted_quantity,
                    req.date,
                    order_date,
                    Self::determine_order_type(config.procurement_type),
                ));

                remaining_inventory += adjusted_quantity;
            }
        }

        Ok(planned_orders)
    }

    /// 週期訂購量（POQ）
    /// 合併指定週期內的需求，一次性訂購
    fn period_order_quantity(
        component_id: &str,
        net_requirements: &[NetRequirement],
        config: &MrpConfig,
        calendar: &mrp_core::WorkCalendar,
    ) -> mrp_core::Result<Vec<PlannedOrder>> {
        // POQ 週期（天數），預設 7 天
        let period_days = 7;

        let mut planned_orders = Vec::new();
        let mut period_start_index = 0;

        while period_start_index < net_requirements.len() {
            let period_start_date = net_requirements[period_start_index].date;

            // 收集週期內的所有需求
            let mut period_total = Decimal::ZERO;
            let mut period_end_index = period_start_index;

            for (idx, req) in net_requirements.iter().enumerate().skip(period_start_index) {
                let days_diff = (req.date - period_start_date).num_days();

                if days_diff < period_days as i64 {
                    period_total += req.net_requirement;
                    period_end_index = idx;
                } else {
                    break;
                }
            }

            // 如果週期內有需求，生成一張訂單
            if period_total > Decimal::ZERO {
                let adjusted_quantity = config.adjust_order_quantity(period_total);
                let order_date =
                    calendar.subtract_working_days(period_start_date, config.lead_time_days);

                planned_orders.push(PlannedOrder::new(
                    component_id.to_string(),
                    adjusted_quantity,
                    period_start_date,
                    order_date,
                    Self::determine_order_type(config.procurement_type),
                ));
            }

            period_start_index = period_end_index + 1;
        }

        Ok(planned_orders)
    }

    /// 最小-最大
    /// 當庫存低於最小值時，補充至最大值
    fn min_max(
        component_id: &str,
        net_requirements: &[NetRequirement],
        config: &MrpConfig,
        calendar: &mrp_core::WorkCalendar,
    ) -> mrp_core::Result<Vec<PlannedOrder>> {
        // Min-Max 使用 minimum_order_qty 作為最小值
        // 使用 maximum_order_qty 作為最大值
        let min_level = config
            .minimum_order_qty
            .unwrap_or(config.safety_stock);

        let max_level = config
            .maximum_order_qty
            .unwrap_or(min_level * Decimal::from(2)); // 預設最大值為最小值的 2 倍

        let mut planned_orders = Vec::new();
        let mut current_inventory = Decimal::ZERO;

        for req in net_requirements {
            current_inventory -= req.gross_requirement;
            current_inventory += req.scheduled_receipt;

            // 如果庫存低於最小值，補充至最大值
            if current_inventory < min_level {
                let order_quantity = max_level - current_inventory;
                let adjusted_quantity = config.adjust_order_quantity(order_quantity);

                let order_date =
                    calendar.subtract_working_days(req.date, config.lead_time_days);

                planned_orders.push(PlannedOrder::new(
                    component_id.to_string(),
                    adjusted_quantity,
                    req.date,
                    order_date,
                    Self::determine_order_type(config.procurement_type),
                ));

                current_inventory += adjusted_quantity;
            }
        }

        Ok(planned_orders)
    }

    /// 決定訂單類型
    fn determine_order_type(procurement_type: ProcurementType) -> PlannedOrderType {
        match procurement_type {
            ProcurementType::Buy => PlannedOrderType::Purchase,
            ProcurementType::Make => PlannedOrderType::Production,
            ProcurementType::Transfer => PlannedOrderType::Transfer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mrp_core::{MrpConfig, ProcurementType, WorkCalendar};
    use rust_decimal::Decimal;

    #[test]
    fn test_lot_for_lot() {
        let calendar = WorkCalendar::default();
        let config = MrpConfig::new("TEST-001".to_string(), 5, ProcurementType::Make);

        let net_reqs = vec![
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                gross_requirement: Decimal::from(100),
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::ZERO,
                net_requirement: Decimal::from(100),
            },
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
                gross_requirement: Decimal::from(50),
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::ZERO,
                net_requirement: Decimal::from(50),
            },
        ];

        let result = LotSizingCalculator::lot_for_lot(
            "TEST-001",
            &net_reqs,
            &config,
            &calendar,
        ).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].quantity, Decimal::from(100));
        assert_eq!(result[1].quantity, Decimal::from(50));
    }

    #[test]
    fn test_fixed_order_quantity() {
        let calendar = WorkCalendar::default();
        let config = MrpConfig::new("TEST-002".to_string(), 3, ProcurementType::Buy)
            .with_lot_sizing_rule(mrp_core::LotSizingRule::FixedOrderQuantity)
            .with_fixed_lot_size(Decimal::from(100));

        let net_reqs = vec![
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                gross_requirement: Decimal::from(150),
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::ZERO,
                net_requirement: Decimal::from(150),
            },
        ];

        let result = LotSizingCalculator::fixed_order_quantity(
            "TEST-002",
            &net_reqs,
            &config,
            &calendar,
        ).unwrap();

        assert!(!result.is_empty());
        // FOQ 應該訂購 200（2批固定批量100）
        assert_eq!(result[0].quantity, Decimal::from(200));
    }

    #[test]
    fn test_period_order_quantity() {
        let calendar = WorkCalendar::default();
        let config = MrpConfig::new("TEST-003".to_string(), 7, ProcurementType::Make)
            .with_lot_sizing_rule(mrp_core::LotSizingRule::PeriodOrderQuantity);

        let net_reqs = vec![
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                gross_requirement: Decimal::ZERO,
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::ZERO,
                net_requirement: Decimal::from(50),
            },
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 3).unwrap(),
                gross_requirement: Decimal::ZERO,
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::ZERO,
                net_requirement: Decimal::from(30),
            },
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(),
                gross_requirement: Decimal::ZERO,
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::ZERO,
                net_requirement: Decimal::from(40),
            },
        ];

        let result = LotSizingCalculator::period_order_quantity(
            "TEST-003",
            &net_reqs,
            &config,
            &calendar,
        ).unwrap();

        // POQ 應該合併 7 天內的需求
        // 11/1 和 11/3 在同一週期（合併為 80）
        // 11/10 在另一週期（40）
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].quantity, Decimal::from(80));
        assert_eq!(result[1].quantity, Decimal::from(40));
    }

    #[test]
    fn test_min_max() {
        let calendar = WorkCalendar::default();
        let config = MrpConfig::new("TEST-004".to_string(), 2, ProcurementType::Buy)
            .with_lot_sizing_rule(mrp_core::LotSizingRule::MinMax)
            .with_minimum_order_qty(Decimal::from(50))
            .with_maximum_order_qty(Decimal::from(200));

        let net_reqs = vec![
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                gross_requirement: Decimal::from(100),
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::from(30),
                net_requirement: Decimal::ZERO,
            },
        ];

        let result = LotSizingCalculator::min_max(
            "TEST-004",
            &net_reqs,
            &config,
            &calendar,
        ).unwrap();

        // 庫存 30 - 需求 100 = -70，低於最小值 50
        // 應該補充到最大值 200
        assert!(!result.is_empty());
        assert!(result[0].quantity <= Decimal::from(200));
    }

    #[test]
    fn test_order_quantity_constraints() {
        let calendar = WorkCalendar::default();
        let config = MrpConfig::new("TEST-005".to_string(), 3, ProcurementType::Make)
            .with_minimum_order_qty(Decimal::from(50))
            .with_maximum_order_qty(Decimal::from(500))
            .with_order_multiple(Decimal::from(25));

        let net_reqs = vec![
            NetRequirement {
                date: NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
                gross_requirement: Decimal::ZERO,
                scheduled_receipt: Decimal::ZERO,
                projected_on_hand: Decimal::ZERO,
                net_requirement: Decimal::from(123), // 應調整為 150（最接近的25倍數）
            },
        ];

        let result = LotSizingCalculator::lot_for_lot(
            "TEST-005",
            &net_reqs,
            &config,
            &calendar,
        ).unwrap();

        assert_eq!(result.len(), 1);
        // 123 調整為 125（滿足最小值50、調整到25的倍數：123→125）
        assert_eq!(result[0].quantity, Decimal::from(125));
    }
}
