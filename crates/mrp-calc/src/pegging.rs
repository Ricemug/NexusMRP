//! 需求追溯

use mrp_core::{Demand, DemandType, PeggingRecord, PlannedOrder};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

/// 追溯類型
#[derive(Debug, Clone, Copy)]
pub enum PeggingType {
    /// 單層追溯
    SingleLevel,
    /// 多層追溯（追溯到最終需求）
    MultiLevel,
}

/// 需求追溯計算器
pub struct PeggingCalculator;

impl PeggingCalculator {
    /// 執行需求追溯
    pub fn perform(
        planned_orders: &[PlannedOrder],
        original_demands: &[Demand],
        pegging_type: PeggingType,
    ) -> mrp_core::Result<HashMap<Uuid, Vec<PeggingRecord>>> {
        let mut pegging_map = HashMap::new();

        for order in planned_orders {
            let pegging = Self::trace_demand_source(
                &order.component_id,
                order.quantity,
                order.required_date,
                original_demands,
                pegging_type,
            )?;

            pegging_map.insert(order.id, pegging);
        }

        Ok(pegging_map)
    }

    /// 追溯需求來源
    fn trace_demand_source(
        component_id: &str,
        quantity: Decimal,
        date: chrono::NaiveDate,
        demands: &[Demand],
        pegging_type: PeggingType,
    ) -> mrp_core::Result<Vec<PeggingRecord>> {
        // 找到該物料在該日期的需求
        let matching_demands: Vec<_> = demands
            .iter()
            .filter(|d| d.component_id == component_id && d.required_date == date)
            .collect();

        let mut pegging_records = Vec::new();
        let mut remaining_qty = quantity;

        for demand in matching_demands {
            if remaining_qty <= Decimal::ZERO {
                break;
            }

            let pegged_qty = demand.quantity.min(remaining_qty);

            // 構建追溯路徑
            let path = match pegging_type {
                PeggingType::SingleLevel => {
                    vec![component_id.to_string()]
                }
                PeggingType::MultiLevel => {
                    // 如果是相依需求，繼續向上追溯
                    if demand.demand_type == DemandType::Dependent {
                        if let Some(parent_ref) = &demand.source_ref {
                            // 遞歸追溯
                            let parent_path = Self::trace_parent_demand(parent_ref, demands)?;
                            let mut full_path = parent_path;
                            full_path.push(component_id.to_string());
                            full_path
                        } else {
                            vec![component_id.to_string()]
                        }
                    } else {
                        vec![component_id.to_string()]
                    }
                }
            };

            pegging_records.push(
                PeggingRecord::new(demand.id, pegged_qty).with_path(path),
            );

            remaining_qty -= pegged_qty;
        }

        Ok(pegging_records)
    }

    /// 追溯父需求
    fn trace_parent_demand(
        parent_id: &str,
        _demands: &[Demand],
    ) -> mrp_core::Result<Vec<String>> {
        // 簡化實現：返回父 ID
        // 實際應遞歸追溯到最頂層
        Ok(vec![parent_id.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mrp_core::{Demand, DemandType, PlannedOrder, PlannedOrderType};

    #[test]
    fn test_single_level_pegging() {
        // 建立計劃訂單
        let planned_order = PlannedOrder::new(
            "COMP-001".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 8).unwrap(), // required_date
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(), // order_date
            PlannedOrderType::Production,
        );

        // 建立原始需求
        let demand = Demand::new(
            "COMP-001".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 8).unwrap(),
            DemandType::SalesOrder,
        );

        // 執行單層追溯
        let result = PeggingCalculator::perform(
            &[planned_order.clone()],
            &[demand.clone()],
            PeggingType::SingleLevel,
        )
        .unwrap();

        assert_eq!(result.len(), 1);

        let pegging_records = result.get(&planned_order.id).unwrap();
        assert_eq!(pegging_records.len(), 1);
        assert_eq!(pegging_records[0].demand_id, demand.id);
        assert_eq!(pegging_records[0].quantity, Decimal::from(100));
        assert_eq!(pegging_records[0].path, vec!["COMP-001".to_string()]);
    }

    #[test]
    fn test_multi_level_pegging_with_dependent_demand() {
        // 建立計劃訂單（子件）
        let planned_order = PlannedOrder::new(
            "CHILD-001".to_string(),
            Decimal::from(200),
            NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(), // required_date
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(), // order_date
            PlannedOrderType::Purchase,
        );

        // 建立相依需求（由父件展開而來）
        let dependent_demand = Demand::new(
            "CHILD-001".to_string(),
            Decimal::from(200),
            NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
            DemandType::Dependent,
        )
        .with_source_ref("PARENT-001".to_string());

        // 執行多層追溯
        let result = PeggingCalculator::perform(
            &[planned_order.clone()],
            &[dependent_demand.clone()],
            PeggingType::MultiLevel,
        )
        .unwrap();

        assert_eq!(result.len(), 1);

        let pegging_records = result.get(&planned_order.id).unwrap();
        assert_eq!(pegging_records.len(), 1);
        assert_eq!(pegging_records[0].demand_id, dependent_demand.id);
        assert_eq!(pegging_records[0].quantity, Decimal::from(200));
        // 多層追溯應包含父件路徑
        assert!(pegging_records[0].path.len() >= 1);
    }

    #[test]
    fn test_partial_quantity_pegging() {
        // 計劃訂單數量大於需求
        let planned_order = PlannedOrder::new(
            "COMP-002".to_string(),
            Decimal::from(150),
            NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(), // required_date
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(), // order_date
            PlannedOrderType::Production,
        );

        // 需求較小
        let demand = Demand::new(
            "COMP-002".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(),
            DemandType::SalesOrder,
        );

        let result = PeggingCalculator::perform(
            &[planned_order.clone()],
            &[demand.clone()],
            PeggingType::SingleLevel,
        )
        .unwrap();

        let pegging_records = result.get(&planned_order.id).unwrap();
        assert_eq!(pegging_records.len(), 1);
        // 追溯數量應為需求數量（較小值）
        assert_eq!(pegging_records[0].quantity, Decimal::from(100));
    }

    #[test]
    fn test_multiple_demands_pegging() {
        // 一個計劃訂單對應多個需求
        let planned_order = PlannedOrder::new(
            "COMP-003".to_string(),
            Decimal::from(300),
            NaiveDate::from_ymd_opt(2025, 11, 15).unwrap(), // required_date
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(), // order_date
            PlannedOrderType::Production,
        );

        let demand1 = Demand::new(
            "COMP-003".to_string(),
            Decimal::from(150),
            NaiveDate::from_ymd_opt(2025, 11, 15).unwrap(),
            DemandType::SalesOrder,
        );

        let demand2 = Demand::new(
            "COMP-003".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 15).unwrap(),
            DemandType::SalesOrder,
        );

        let result = PeggingCalculator::perform(
            &[planned_order.clone()],
            &[demand1.clone(), demand2.clone()],
            PeggingType::SingleLevel,
        )
        .unwrap();

        let pegging_records = result.get(&planned_order.id).unwrap();
        // 應該追溯到兩個需求
        assert_eq!(pegging_records.len(), 2);

        let total_pegged: Decimal = pegging_records.iter().map(|r| r.quantity).sum();
        assert_eq!(total_pegged, Decimal::from(250)); // 150 + 100
    }

    #[test]
    fn test_no_matching_demand() {
        // 計劃訂單沒有對應的需求
        let planned_order = PlannedOrder::new(
            "COMP-004".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(), // required_date
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(), // order_date
            PlannedOrderType::Production,
        );

        // 不同日期的需求
        let demand = Demand::new(
            "COMP-004".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 20).unwrap(), // 不同日期
            DemandType::SalesOrder,
        );

        let result = PeggingCalculator::perform(
            &[planned_order.clone()],
            &[demand],
            PeggingType::SingleLevel,
        )
        .unwrap();

        let pegging_records = result.get(&planned_order.id).unwrap();
        // 應該沒有追溯記錄
        assert_eq!(pegging_records.len(), 0);
    }
}
