//! 腳踏車 MRP 計算完整範例
//!
//! 展示從需求到計劃訂單的完整 MRP 計算流程

use chrono::NaiveDate;
use mrp_calc::MrpCalculator;
use mrp_core::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("===== Bike MRP Calculation Example =====\n");

    // 步驟 1: 建立工廠工作日曆（24/7 模式）
    println!("[1] Create Factory Calendar");
    let calendar = WorkCalendar::fallback_calendar();
    println!("    Calendar: 24/7 (Fallback mode)\n");

    // 步驟 2: 建立 BOM 結構
    println!("[2] Create BOM Structure");
    let bom_graph = create_bike_bom()?;
    println!("    Nodes: {}", bom_graph.arena().node_count());
    println!("    Edges: {}\n", bom_graph.arena().edge_count());

    // 步驟 3: 設定 MRP 參數
    println!("[3] Configure MRP Parameters");
    let mut configs = HashMap::new();

    // Bike (LFL)
    configs.insert(
        "BIKE-001".to_string(),
        MrpConfig::new("BIKE-001".to_string(), 7, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::LotForLot)
            .with_safety_stock(Decimal::from(5)),
    );
    println!("    BIKE-001: Lot-for-Lot, Lead Time 7 days");

    // Frame (FOQ)
    configs.insert(
        "FRAME-001".to_string(),
        MrpConfig::new("FRAME-001".to_string(), 5, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::FixedOrderQuantity)
            .with_fixed_lot_size(Decimal::from(100))
            .with_safety_stock(Decimal::from(10)),
    );
    println!("    FRAME-001: FOQ 100, Lead Time 5 days");

    // Wheel (POQ)
    configs.insert(
        "WHEEL-001".to_string(),
        MrpConfig::new("WHEEL-001".to_string(), 3, ProcurementType::Buy)
            .with_lot_sizing_rule(LotSizingRule::PeriodOrderQuantity)
            .with_minimum_order_qty(Decimal::from(200))
            .with_safety_stock(Decimal::from(20)),
    );
    println!("    WHEEL-001: POQ, Lead Time 3 days\n");

    // 步驟 4: 建立需求
    println!("[4] Create Demands");
    let demands = vec![
        Demand::new(
            "BIKE-001".to_string(),
            Decimal::from(150),
            NaiveDate::from_ymd_opt(2025, 11, 15).unwrap(),
            DemandType::SalesOrder,
        )
        .with_source_ref("SO-001".to_string()),
        Demand::new(
            "BIKE-001".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 22).unwrap(),
            DemandType::SalesOrder,
        )
        .with_source_ref("SO-002".to_string()),
    ];

    println!("    SO-001: 150 bikes on 2025-11-15");
    println!("    SO-002: 100 bikes on 2025-11-22\n");

    // 步驟 5: 現有供應
    println!("[5] Existing Supplies");
    let supplies = vec![Supply::new(
        "FRAME-001".to_string(),
        Decimal::from(50),
        NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(),
        SupplyType::PurchaseOrder,
    )
    .with_source_ref("PO-100".to_string())
    .as_firm()];

    println!("    PO-100: 50 frames on 2025-11-10\n");

    // 步驟 6: 現有庫存
    println!("[6] Current Inventory");
    let inventories = vec![
        Inventory::new("BIKE-001".to_string(), Decimal::from(10), Decimal::from(5)),
        Inventory::new(
            "FRAME-001".to_string(),
            Decimal::from(30),
            Decimal::from(10),
        ),
        Inventory::new(
            "WHEEL-001".to_string(),
            Decimal::from(100),
            Decimal::from(20),
        ),
    ];

    println!("    BIKE-001: On-hand 10, Safety 5");
    println!("    FRAME-001: On-hand 30, Safety 10");
    println!("    WHEEL-001: On-hand 100, Safety 20\n");

    // 步驟 7: 執行 MRP
    println!("[7] Execute MRP Calculation");
    let calculator = MrpCalculator::new(bom_graph, configs, calendar);
    let result = calculator.calculate(demands, supplies, inventories)?;

    println!("    Completed in {} ms\n", result.calculation_time_ms.unwrap_or(0));

    // 步驟 8: 顯示結果
    println!("[8] MRP Results");
    println!("    Total Planned Orders: {}\n", result.planned_orders.len());

    // 按物料分組
    let mut orders_by_component: HashMap<String, Vec<&PlannedOrder>> = HashMap::new();
    for order in &result.planned_orders {
        orders_by_component
            .entry(order.component_id.clone())
            .or_insert_with(Vec::new)
            .push(order);
    }

    for (component_id, orders) in orders_by_component.iter() {
        println!("    Component: {}", component_id);
        for order in orders {
            println!(
                "      - Type: {:?} | Qty: {} | Order: {} | Required: {}",
                order.order_type, order.quantity, order.order_date, order.required_date
            );
        }
        println!();
    }

    if !result.warnings.is_empty() {
        println!("    Warnings:");
        for warning in &result.warnings {
            println!("      - [{}] {}", warning.component_id, warning.message);
        }
    }

    println!("===== MRP Calculation Complete =====\n");

    Ok(())
}

/// 建立腳踏車 BOM
fn create_bike_bom() -> std::result::Result<bom_graph::BomGraph, Box<dyn std::error::Error>> {
    use bom_core::{BomItem, ComponentId};
    use uuid::Uuid;

    let mut graph = bom_graph::BomGraph::new();

    // BIKE = 1 FRAME + 2 WHEEL
    let bike_frame = BomItem {
        id: Uuid::new_v4(),
        parent_id: ComponentId::new("BIKE-001"),
        child_id: ComponentId::new("FRAME-001"),
        quantity: Decimal::from(1),
        scrap_factor: Decimal::ZERO,
        sequence: 10,
        operation_sequence: None,
        is_phantom: false,
        effective_from: None,
        effective_to: None,
        alternative_group: None,
        alternative_priority: None,
        reference_designator: None,
        position: None,
        notes: None,
        version: 1,
    };

    let bike_wheel = BomItem {
        id: Uuid::new_v4(),
        parent_id: ComponentId::new("BIKE-001"),
        child_id: ComponentId::new("WHEEL-001"),
        quantity: Decimal::from(2),
        scrap_factor: Decimal::ZERO,
        sequence: 20,
        operation_sequence: None,
        is_phantom: false,
        effective_from: None,
        effective_to: None,
        alternative_group: None,
        alternative_priority: None,
        reference_designator: None,
        position: None,
        notes: None,
        version: 1,
    };

    graph.add_bom_item(bike_frame)?;
    graph.add_bom_item(bike_wheel)?;

    Ok(graph)
}
