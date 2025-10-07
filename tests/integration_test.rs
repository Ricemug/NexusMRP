//! 集成測試

use chrono::NaiveDate;
use mrp_calc::MrpCalculator;
use mrp_core::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[test]
fn test_single_level_bom_mrp() {
    // 測試單層 BOM MRP 計算
    // 場景：Product-A 需要 2 個 Part-B

    // 1. 建立 BOM
    let mut bom = bom_graph::BomGraph::new();
    let bom_item = bom_core::BomItem {
        id: uuid::Uuid::new_v4(),
        parent_id: bom_core::ComponentId::new("PRODUCT-A"),
        child_id: bom_core::ComponentId::new("PART-B"),
        quantity: Decimal::from(2),
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
    bom.add_bom_item(bom_item).unwrap();

    // Debug: Check BOM structure
    println!("BOM nodes: {}", bom.arena().node_count());
    println!("BOM edges: {}", bom.arena().edge_count());

    // 2. MRP 配置
    let mut configs = HashMap::new();
    configs.insert(
        "PRODUCT-A".to_string(),
        MrpConfig::new("PRODUCT-A".to_string(), 5, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::LotForLot),
    );
    configs.insert(
        "PART-B".to_string(),
        MrpConfig::new("PART-B".to_string(), 3, ProcurementType::Buy)
            .with_lot_sizing_rule(LotSizingRule::LotForLot),
    );

    // 3. 工作日曆 (24/7)
    let calendar = WorkCalendar::fallback_calendar();

    // 4. 需求：100 個 Product-A 在 11/20
    let demands = vec![Demand::new(
        "PRODUCT-A".to_string(),
        Decimal::from(100),
        NaiveDate::from_ymd_opt(2025, 11, 20).unwrap(),
        DemandType::SalesOrder,
    )];

    // 5. 無現有供應和庫存
    let supplies = vec![];
    let inventories = vec![];

    // 6. 執行 MRP
    let calculator = MrpCalculator::new(bom, configs, calendar);
    let result = calculator.calculate(demands, supplies, inventories).unwrap();

    // 7. 驗證結果
    println!("Total planned orders: {}", result.planned_orders.len());
    for order in &result.planned_orders {
        println!("  - {} qty {}", order.component_id, order.quantity);
    }

    assert!(result.planned_orders.len() >= 2, "Expected >= 2 orders, got {}", result.planned_orders.len()); // Product-A 和 Part-B 各至少一個訂單

    // 找到 Product-A 的計劃訂單
    let product_a_orders: Vec<_> = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "PRODUCT-A")
        .collect();
    assert_eq!(product_a_orders.len(), 1);
    assert_eq!(product_a_orders[0].quantity, Decimal::from(100));
    // 訂單日應該是需求日 - 提前期 (5天) = 11/15
    assert_eq!(
        product_a_orders[0].order_date,
        NaiveDate::from_ymd_opt(2025, 11, 15).unwrap()
    );

    // 找到 Part-B 的計劃訂單
    let part_b_orders: Vec<_> = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "PART-B")
        .collect();
    assert!(part_b_orders.len() >= 1);

    // Part-B 數量應該是 100 * 2 = 200（因為 BOM 中每個 Product-A 需要 2 個 Part-B）
    let total_part_b: Decimal = part_b_orders.iter().map(|o| o.quantity).sum();
    assert_eq!(total_part_b, Decimal::from(200));
}

#[test]
fn test_multi_level_bom_mrp() {
    // 測試多層 BOM MRP 計算
    // 場景：
    //   Bike (腳踏車)
    //     ├── Frame (車架) x1
    //     │   └── Steel-Tube (鋼管) x3
    //     └── Wheel (輪子) x2

    // 1. 建立三層 BOM
    let mut bom = bom_graph::BomGraph::new();

    // Bike -> Frame
    bom.add_bom_item(bom_core::BomItem {
        id: uuid::Uuid::new_v4(),
        parent_id: bom_core::ComponentId::new("BIKE"),
        child_id: bom_core::ComponentId::new("FRAME"),
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
    })
    .unwrap();

    // Bike -> Wheel
    bom.add_bom_item(bom_core::BomItem {
        id: uuid::Uuid::new_v4(),
        parent_id: bom_core::ComponentId::new("BIKE"),
        child_id: bom_core::ComponentId::new("WHEEL"),
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
    })
    .unwrap();

    // Frame -> Steel-Tube
    bom.add_bom_item(bom_core::BomItem {
        id: uuid::Uuid::new_v4(),
        parent_id: bom_core::ComponentId::new("FRAME"),
        child_id: bom_core::ComponentId::new("STEEL-TUBE"),
        quantity: Decimal::from(3),
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
    })
    .unwrap();

    // 2. MRP 配置
    let mut configs = HashMap::new();
    configs.insert(
        "BIKE".to_string(),
        MrpConfig::new("BIKE".to_string(), 7, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::LotForLot),
    );
    configs.insert(
        "FRAME".to_string(),
        MrpConfig::new("FRAME".to_string(), 5, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::LotForLot),
    );
    configs.insert(
        "WHEEL".to_string(),
        MrpConfig::new("WHEEL".to_string(), 3, ProcurementType::Buy)
            .with_lot_sizing_rule(LotSizingRule::LotForLot),
    );
    configs.insert(
        "STEEL-TUBE".to_string(),
        MrpConfig::new("STEEL-TUBE".to_string(), 2, ProcurementType::Buy)
            .with_lot_sizing_rule(LotSizingRule::LotForLot),
    );

    // 3. 工作日曆
    let calendar = WorkCalendar::fallback_calendar();

    // 4. 需求：50 輛腳踏車在 12/1
    let demands = vec![Demand::new(
        "BIKE".to_string(),
        Decimal::from(50),
        NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
        DemandType::SalesOrder,
    )];

    // 5. 無現有供應和庫存
    let supplies = vec![];
    let inventories = vec![];

    // 6. 執行 MRP
    let calculator = MrpCalculator::new(bom, configs, calendar);
    let result = calculator.calculate(demands, supplies, inventories).unwrap();

    // 7. 驗證多層展開結果
    // 應該有 4 個物料的計劃訂單
    let bike_qty: Decimal = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "BIKE")
        .map(|o| o.quantity)
        .sum();
    assert_eq!(bike_qty, Decimal::from(50));

    let frame_qty: Decimal = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "FRAME")
        .map(|o| o.quantity)
        .sum();
    assert_eq!(frame_qty, Decimal::from(50)); // 50 bikes * 1 frame = 50

    let wheel_qty: Decimal = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "WHEEL")
        .map(|o| o.quantity)
        .sum();
    assert_eq!(wheel_qty, Decimal::from(100)); // 50 bikes * 2 wheels = 100

    let steel_qty: Decimal = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "STEEL-TUBE")
        .map(|o| o.quantity)
        .sum();
    assert_eq!(steel_qty, Decimal::from(150)); // 50 frames * 3 tubes = 150
}

#[test]
fn test_lot_sizing_rules_integration() {
    // 測試不同批量規則的集成
    // 場景：相同需求，不同批量規則產生不同計劃訂單

    let calendar = WorkCalendar::fallback_calendar();

    let demands = vec![
        Demand::new(
            "PART-X".to_string(),
            Decimal::from(75),
            NaiveDate::from_ymd_opt(2025, 11, 15).unwrap(),
            DemandType::SalesOrder,
        ),
        Demand::new(
            "PART-X".to_string(),
            Decimal::from(45),
            NaiveDate::from_ymd_opt(2025, 11, 22).unwrap(),
            DemandType::SalesOrder,
        ),
    ];

    let supplies = vec![];
    let inventories = vec![];

    // 測試 1: Lot-for-Lot (批對批)
    {
        let bom = bom_graph::BomGraph::new(); // 空 BOM，只測試單物料
        let mut configs = HashMap::new();
        configs.insert(
            "PART-X".to_string(),
            MrpConfig::new("PART-X".to_string(), 3, ProcurementType::Buy)
                .with_lot_sizing_rule(LotSizingRule::LotForLot),
        );

        let calculator = MrpCalculator::new(bom, configs, calendar.clone());
        let result = calculator
            .calculate(demands.clone(), supplies.clone(), inventories.clone())
            .unwrap();

        // 應該有 2 個訂單，數量分別為 75 和 45
        let orders: Vec<_> = result
            .planned_orders
            .iter()
            .filter(|o| o.component_id == "PART-X")
            .collect();
        assert_eq!(orders.len(), 2);
    }

    // 測試 2: Fixed Order Quantity (固定批量 100)
    {
        let bom = bom_graph::BomGraph::new();
        let mut configs = HashMap::new();
        configs.insert(
            "PART-X".to_string(),
            MrpConfig::new("PART-X".to_string(), 3, ProcurementType::Buy)
                .with_lot_sizing_rule(LotSizingRule::FixedOrderQuantity)
                .with_fixed_lot_size(Decimal::from(100)),
        );

        let calculator = MrpCalculator::new(bom, configs, calendar.clone());
        let result = calculator
            .calculate(demands.clone(), supplies.clone(), inventories.clone())
            .unwrap();

        // 應該有 2 個訂單，每個都是 100
        let orders: Vec<_> = result
            .planned_orders
            .iter()
            .filter(|o| o.component_id == "PART-X")
            .collect();
        assert_eq!(orders.len(), 2);
        assert!(orders.iter().all(|o| o.quantity == Decimal::from(100)));
    }

    // 測試 3: Minimum Order Quantity (最小訂購量 200)
    {
        let bom = bom_graph::BomGraph::new();
        let mut configs = HashMap::new();
        configs.insert(
            "PART-X".to_string(),
            MrpConfig::new("PART-X".to_string(), 3, ProcurementType::Buy)
                .with_lot_sizing_rule(LotSizingRule::LotForLot)
                .with_minimum_order_qty(Decimal::from(200)),
        );

        let calculator = MrpCalculator::new(bom, configs, calendar.clone());
        let result = calculator
            .calculate(demands.clone(), supplies.clone(), inventories.clone())
            .unwrap();

        // 第一個訂單應該是 200（75 調整到最小量）
        let first_order = result
            .planned_orders
            .iter()
            .find(|o| o.component_id == "PART-X" && o.required_date == NaiveDate::from_ymd_opt(2025, 11, 15).unwrap())
            .unwrap();
        assert!(first_order.quantity >= Decimal::from(200));
    }
}

#[test]
fn test_with_existing_inventory_and_supply() {
    // 測試有現有庫存和供應的情況

    let mut bom = bom_graph::BomGraph::new();
    bom.add_bom_item(bom_core::BomItem {
        id: uuid::Uuid::new_v4(),
        parent_id: bom_core::ComponentId::new("PRODUCT"),
        child_id: bom_core::ComponentId::new("COMPONENT"),
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
    })
    .unwrap();

    let mut configs = HashMap::new();
    configs.insert(
        "PRODUCT".to_string(),
        MrpConfig::new("PRODUCT".to_string(), 5, ProcurementType::Make)
            .with_lot_sizing_rule(LotSizingRule::LotForLot)
            .with_safety_stock(Decimal::from(10)),
    );
    configs.insert(
        "COMPONENT".to_string(),
        MrpConfig::new("COMPONENT".to_string(), 3, ProcurementType::Buy)
            .with_lot_sizing_rule(LotSizingRule::LotForLot)
            .with_safety_stock(Decimal::from(5)),
    );

    let calendar = WorkCalendar::fallback_calendar();

    // 需求：100 個產品
    let demands = vec![Demand::new(
        "PRODUCT".to_string(),
        Decimal::from(100),
        NaiveDate::from_ymd_opt(2025, 11, 20).unwrap(),
        DemandType::SalesOrder,
    )];

    // 現有庫存：30 個產品，20 個組件
    let inventories = vec![
        Inventory::new(
            "PRODUCT".to_string(),
            Decimal::from(30),
            Decimal::from(10),
        ),
        Inventory::new(
            "COMPONENT".to_string(),
            Decimal::from(20),
            Decimal::from(5),
        ),
    ];

    // 現有供應：10 個組件將在 11/18 到貨
    let supplies = vec![Supply::new(
        "COMPONENT".to_string(),
        Decimal::from(10),
        NaiveDate::from_ymd_opt(2025, 11, 18).unwrap(),
        SupplyType::PurchaseOrder,
    )];

    let calculator = MrpCalculator::new(bom, configs, calendar);
    let result = calculator.calculate(demands, supplies, inventories).unwrap();

    // 產品計劃訂單應該考慮現有庫存
    // 需求 100 - 庫存 30 + 安全庫存 10 = 80
    let product_orders: Vec<_> = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "PRODUCT")
        .collect();

    assert!(product_orders.len() >= 1);
    let total_product: Decimal = product_orders.iter().map(|o| o.quantity).sum();
    assert_eq!(total_product, Decimal::from(80));

    // 組件應該考慮現有庫存和供應
    let component_orders: Vec<_> = result
        .planned_orders
        .iter()
        .filter(|o| o.component_id == "COMPONENT")
        .collect();

    // 有計劃訂單（因為需要覆蓋產品的需求，扣除庫存和供應後仍有缺口）
    assert!(component_orders.len() >= 1);
}
