//! # 腳踏車 MRP 計算完整範例
//!
//! 這個範例展示完整的 MRP 計算流程：
//! - 產品：腳踏車
//! - 零件：車架、輪子、座椅
//! - 需求：銷售訂單
//! - 批量規則：不同零件使用不同策略

use chrono::NaiveDate;
use mrp_calc::MrpCalculator;
use mrp_core::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("🚲 ===== 腳踏車 MRP 計算範例 =====\");
    println!();

    // ========== 1. 建立工作日曆 ==========
    println!("📅 步驟 1: 建立工廠工作日曆");

    // 選項 A: 從排班表載入
    let calendar = if let Some(shift_data) = load_shift_schedule() {
        println!("   ✓ 從排班表載入：{}", shift_data.calendar_id);
        WorkCalendar::from_shift_data(
            shift_data.calendar_id,
            shift_data.working_days,
            shift_data.holidays,
        )
    } else {
        // 選項 B: 降級使用 24/7
        println!("   ⚠ 無排班表，使用 24/7 日曆");
        WorkCalendar::fallback_calendar()
    };

    println!("   工作模式: {:?}", calendar.working_days);
    println!();

    // ========== 2. 建立產品 BOM 結構 ==========
    println!("🔧 步驟 2: 建立 BOM 結構");
    let bom_graph = create_bike_bom()?;
    println!("   ✓ BOM 節點數: {}", bom_graph.arena().node_count());
    println!("   ✓ BOM 邊數: {}", bom_graph.arena().edge_count());
    println!();

    // ========== 3. 設定 MRP 參數 ==========
    println!("⚙️  步驟 3: 設定各物料 MRP 參數");
    let mut configs = HashMap::new();

    // 腳踏車（成品）- 批對批
    configs.insert(
        "BIKE-001".to_string(),
        MrpConfig::new("BIKE-001".to_string(), 7, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::LotForLot)
            .with_safety_stock(Decimal::from(5)),
    );
    println!("   ✓ BIKE-001: 批對批 (LFL), 提前期 7 天");

    // 車架 - 固定批量100
    configs.insert(
        "FRAME-001".to_string(),
        MrpConfig::new("FRAME-001".to_string(), 5, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::FixedOrderQuantity)
            .with_fixed_lot_size(Decimal::from(100))
            .with_safety_stock(Decimal::from(10)),
    );
    println!("   ✓ FRAME-001: 固定批量 (FOQ) 100, 提前期 5 天");

    // 輪子 - 週期訂購量
    configs.insert(
        "WHEEL-001".to_string(),
        MrpConfig::new("WHEEL-001".to_string(), 3, ProcurementType::Buy)
            .with_lot_sizing_rule(LotSizingRule::PeriodOrderQuantity)
            .with_minimum_order_qty(Decimal::from(200))
            .with_order_multiple(Decimal::from(50))
            .with_safety_stock(Decimal::from(20)),
    );
    println!("   ✓ WHEEL-001: 週期訂購 (POQ), 提前期 3 天");
    println!();

    // ========== 4. 建立需求 ==========
    println!("📦 步驟 4: 建立銷售訂單需求");
    let demands = vec![
        Demand::new(
            "BIKE-001".to_string(),
            Decimal::from(150),
            NaiveDate::from_ymd_opt(2025, 11, 15).unwrap(),
            DemandType::SalesOrder,
        )
        .with_source_ref("SO-2025-001".to_string())
        .with_priority(8),

        Demand::new(
            "BIKE-001".to_string(),
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2025, 11, 22).unwrap(),
            DemandType::SalesOrder,
        )
        .with_source_ref("SO-2025-002".to_string())
        .with_priority(5),
    ];

    println!("   ✓ SO-2025-001: 150 台腳踏車，需求日 11/15");
    println!("   ✓ SO-2025-002: 100 台腳踏車，需求日 11/22");
    println!();

    // ========== 5. 建立現有供應 ==========
    println!("📥 步驟 5: 現有採購訂單");
    let supplies = vec![
        Supply::new(
            "FRAME-001".to_string(),
            Decimal::from(50),
            NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(),
            SupplyType::PurchaseOrder,
        )
        .with_source_ref("PO-2025-100".to_string())
        .as_firm(),
    ];

    println!("   ✓ PO-2025-100: 50 個車架，到貨日 11/10");
    println!();

    // ========== 6. 建立現有庫存 ==========
    println!("📊 步驟 6: 當前庫存狀態");
    let inventories = vec![
        Inventory::new(
            "BIKE-001".to_string(),
            Decimal::from(10),
            Decimal::from(5),
        ),
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

    println!("   ✓ BIKE-001: 現有 10, 安全庫存 5");
    println!("   ✓ FRAME-001: 現有 30, 安全庫存 10");
    println!("   ✓ WHEEL-001: 現有 100, 安全庫存 20");
    println!();

    // ========== 7. 執行 MRP 計算 ==========
    println!("🚀 步驟 7: 執行 MRP 計算");
    println!("   計算中...");

    let calculator = MrpCalculator::new(bom_graph, configs, calendar);
    let result = calculator.calculate(demands, supplies, inventories)?;

    println!("   ✓ 完成！耗時 {} ms", result.calculation_time_ms.unwrap_or(0));
    println!();

    // ========== 8. 顯示結果 ==========
    println!("📋 步驟 8: MRP 計算結果");
    println!("----------------------------------------");
    println!("計劃訂單總數: {}", result.planned_orders.len());
    println!();

    // 按物料分組顯示
    let mut orders_by_component: HashMap<String, Vec<&PlannedOrder>> = HashMap::new();
    for order in &result.planned_orders {
        orders_by_component
            .entry(order.component_id.clone())
            .or_insert_with(Vec::new)
            .push(order);
    }

    for (component_id, orders) in orders_by_component.iter() {
        println!("物料: {}", component_id);
        for order in orders {
            println!(
                "  ├─ {:?} | 數量: {} | 訂單日: {} | 需求日: {}",
                order.order_type,
                order.quantity,
                order.order_date,
                order.required_date
            );
        }
        println!();
    }

    // ========== 9. 警告訊息 ==========
    if !result.warnings.is_empty() {
        println!("⚠️  警告訊息:");
        for warning in &result.warnings {
            println!("  - [{}] {}", warning.component_id, warning.message);
        }
        println!();
    }

    println!("✅ MRP 計算完成！");
    println!();

    Ok(())
}

/// 模擬從 ERP 系統載入排班表
fn load_shift_schedule() -> Option<ShiftSchedule> {
    // 實際應用：從數據庫或 API 取得
    // 這裡模擬週一到週六上班的工廠
    Some(ShiftSchedule {
        calendar_id: "FACTORY-A".to_string(),
        working_days: vec![true, true, true, true, true, true, false], // 週六上班
        holidays: vec![],
    })
}

/// 建立腳踏車 BOM 結構
fn create_bike_bom() -> Result<bom_graph::BomGraph> {
    use bom_core::{BomItem, ComponentId};

    let mut graph = bom_graph::BomGraph::new();

    // 腳踏車 = 1 個車架 + 2 個輪子
    let bike_frame = BomItem {
        parent_id: ComponentId::new("BIKE-001"),
        child_id: ComponentId::new("FRAME-001"),
        quantity: Decimal::from(1),
        unit: "EA".to_string(),
        effective_date: None,
        expiration_date: None,
    };

    let bike_wheel = BomItem {
        parent_id: ComponentId::new("BIKE-001"),
        child_id: ComponentId::new("WHEEL-001"),
        quantity: Decimal::from(2),
        unit: "EA".to_string(),
        effective_date: None,
        expiration_date: None,
    };

    graph.add_bom_item(bike_frame)?;
    graph.add_bom_item(bike_wheel)?;

    Ok(graph)
}
