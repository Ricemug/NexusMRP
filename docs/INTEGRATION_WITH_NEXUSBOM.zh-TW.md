# 🔗 整合指南：NexusMRP + NexusBom

> 物料需求計劃與物料清單整合完整指南

[English](./INTEGRATION_WITH_NEXUSBOM.md) | [简体中文](./INTEGRATION_WITH_NEXUSBOM.zh-CN.md) | [Deutsch](./INTEGRATION_WITH_NEXUSBOM.de.md)

本指南說明如何整合 **NexusMRP**（物料需求計劃）與 **NexusBom**（物料清單）以建構完整的製造規劃系統。

## 📋 目錄

- [概述](#概述)
- [為什麼要整合？](#為什麼要整合)
- [架構](#架構)
- [整合步驟](#整合步驟)
- [程式碼範例](#程式碼範例)
- [最佳實踐](#最佳實踐)
- [疑難排解](#疑難排解)

## 概述

**NexusBom** 和 **NexusMRP** 被設計為互補系統：

- **NexusBom**：管理產品結構、物料展開和成本計算
- **NexusMRP**：規劃物料需求、排程生產和管理庫存

兩者結合形成強大的製造規劃解決方案。

## 為什麼要整合？

| 未整合 | 已整合 |
|--------|--------|
| 手動 BOM 查詢 | 自動物料展開 |
| 靜態規劃 | 動態需求傳播 |
| 系統分離 | 端到端可視化 |
| 有限優化 | 產能感知規劃 |

### 主要優勢

✅ **自動多階規劃** - MRP 使用 BOM 將需求展開至所有階層
✅ **即時成本分析** - 結合計劃訂單與 BOM 成本
✅ **變更影響分析** - 查看 BOM 變更如何影響物料計劃
✅ **虛設件處理** - MRP 遵循 BOM 虛設零件規則
✅ **替代 BOM 支援** - 以不同製造路徑進行規劃

## 架構

```
┌─────────────────────────────────────────────────────────┐
│                   您的應用程式                           │
└─────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┴───────────────┐
           │                               │
           ▼                               ▼
┌──────────────────────┐       ┌──────────────────────┐
│     NexusBom         │       │     NexusMRP         │
│   (BOM 結構)         │◄──────│   (規劃邏輯)         │
└──────────────────────┘       └──────────────────────┘
           │                               │
           │     物料展開                   │
           │     零件清單                   │
           │     成本資料                   │
           └───────────────────────────────┘
```

### 資料流程

1. **載入 BOM 資料** → NexusBom 建立產品結構圖
2. **建立需求** → NexusMRP 接收頂層需求
3. **展開 BOM** → NexusBom 提供零件清單與數量
4. **計算 MRP** → NexusMRP 透過 BOM 階層傳播需求
5. **產生計劃** → 輸出所有零件的計劃訂單

## 整合步驟

### 步驟 1：新增相依套件

在 `Cargo.toml` 中新增兩個函式庫：

```toml
[dependencies]
# NexusBom - BOM 計算引擎
bom-core = { git = "https://github.com/Ricemug/NexusBom" }
bom-calc = { git = "https://github.com/Ricemug/NexusBom" }
bom-graph = { git = "https://github.com/Ricemug/NexusBom" }

# NexusMRP - MRP 計算引擎
mrp-core = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-calc = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-cache = { git = "https://github.com/Ricemug/NexusMRP" }
```

### 步驟 2：建立 BOM 圖

```rust
use bom_core::*;
use bom_graph::BomGraph;

// 定義產品結構
let components = vec![
    Component {
        id: ComponentId::new("BIKE-001"),
        description: "完整腳踏車".to_string(),
        component_type: ComponentType::FinishedProduct,
        standard_cost: Some(Decimal::new(50000, 2)), // $500
        lead_time_days: 5,
        procurement_type: ProcurementType::Make,
    },
    Component {
        id: ComponentId::new("FRAME-001"),
        description: "腳踏車車架".to_string(),
        component_type: ComponentType::SubAssembly,
        standard_cost: Some(Decimal::new(20000, 2)), // $200
        lead_time_days: 10,
        procurement_type: ProcurementType::Buy,
    },
    // ... 更多零件
];

let bom_items = vec![
    BomItem {
        parent_id: ComponentId::new("BIKE-001"),
        child_id: ComponentId::new("FRAME-001"),
        quantity: Decimal::ONE,
        sequence: 10,
        is_phantom: false,
    },
    // ... 更多 BOM 關係
];

// 建立 BOM 圖
let bom_graph = BomGraph::from_components(&components, &bom_items)?;
```

### 步驟 3：執行物料展開

```rust
use bom_calc::ExplosionCalculator;

// 針對特定數量展開 BOM
let explosion_calc = ExplosionCalculator::new(&bom_graph);
let explosion_result = explosion_calc.explode(
    &ComponentId::new("BIKE-001"),
    Decimal::from(100), // 數量：100 台腳踏車
)?;

// 取得扁平化的零件需求
let component_requirements = explosion_result.get_flattened_requirements();
```

### 步驟 4：從 BOM 建立 MRP 需求

```rust
use mrp_core::*;
use chrono::NaiveDate;

// 將 BOM 展開轉換為 MRP 需求
let due_date = NaiveDate::from_ymd_opt(2025, 12, 1).unwrap();
let mut demands = Vec::new();

for (component_id, total_qty) in component_requirements {
    let demand = Demand::new(
        component_id.to_string(),
        total_qty,
        due_date,
        DemandType::ProductionOrder,
    );
    demands.push(demand);
}
```

### 步驟 5：使用 BOM 提前期配置 MRP

```rust
use mrp_calc::MRPCalculator;

// 使用 BOM 資料建立 MRP 配置
let mut mrp_configs = Vec::new();

for component in &components {
    let config = MrpConfig {
        item_id: component.id.to_string(),
        lead_time_days: component.lead_time_days,
        procurement_type: component.procurement_type.clone(),
        lot_sizing_rule: LotSizingRule::LotForLot,
        minimum_order_qty: None,
        maximum_order_qty: None,
        order_multiple: None,
        safety_stock: Decimal::ZERO,
    };
    mrp_configs.push(config);
}
```

### 步驟 6：執行整合 MRP 計算

```rust
// 初始化 MRP 計算器
let mrp_calculator = MRPCalculator::new(mrp_configs);

// 使用基於 BOM 的需求執行 MRP
let mrp_result = mrp_calculator.calculate(
    &demands,
    &existing_supplies,    // 任何現有採購單或生產單
    &inventory_balances,   // 目前庫存
)?;

// 取得計劃訂單
let planned_orders = mrp_result.planned_orders;

println!("產生 {} 筆計劃訂單", planned_orders.len());
for order in planned_orders {
    println!("  {} - 數量: {} - 日期: {}",
        order.item_id, order.quantity, order.due_date);
}
```

## 程式碼範例

### 完整整合範例

```rust
use bom_core::*;
use bom_graph::BomGraph;
use bom_calc::ExplosionCalculator;
use mrp_core::*;
use mrp_calc::MRPCalculator;
use rust_decimal::Decimal;
use chrono::NaiveDate;

fn integrated_planning_example() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 建立 BOM 結構
    let bom_graph = build_bicycle_bom()?;

    // 2. 接收客戶訂單
    let customer_order = CustomerOrder {
        product_id: "BIKE-001".to_string(),
        quantity: Decimal::from(100),
        due_date: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
    };

    // 3. 展開 BOM 以取得零件需求
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    let explosion = explosion_calc.explode_with_lead_time_offset(
        &ComponentId::new(&customer_order.product_id),
        customer_order.quantity,
        customer_order.due_date,
    )?;

    // 4. 轉換為 MRP 需求
    let demands: Vec<Demand> = explosion
        .items
        .iter()
        .map(|item| Demand {
            item_id: item.component_id.to_string(),
            quantity: item.total_quantity,
            due_date: item.required_date,
            demand_type: DemandType::DependentDemand,
            source_id: Some(customer_order.product_id.clone()),
        })
        .collect();

    // 5. 執行 MRP 計算
    let mrp_configs = extract_mrp_configs_from_bom(&bom_graph);
    let calculator = MRPCalculator::new(mrp_configs);

    let mrp_result = calculator.calculate(
        &demands,
        &vec![], // 無現有供應
        &vec![], // 無現有庫存
    )?;

    // 6. 輸出計劃訂單
    println!("客戶訂單 {} 的計劃訂單：", customer_order.product_id);
    for order in mrp_result.planned_orders {
        println!("  訂單: {} - 數量: {} - 開始: {} - 到期: {}",
            order.item_id,
            order.quantity,
            order.order_date,
            order.due_date
        );
    }

    // 7. 使用 BOM 計算總成本
    let total_cost = calculate_order_cost(&bom_graph, &mrp_result.planned_orders)?;
    println!("總物料成本: ${:.2}", total_cost);

    Ok(())
}

fn extract_mrp_configs_from_bom(bom_graph: &BomGraph) -> Vec<MrpConfig> {
    bom_graph
        .get_all_components()
        .iter()
        .map(|component| MrpConfig {
            item_id: component.id.to_string(),
            lead_time_days: component.lead_time_days,
            procurement_type: component.procurement_type.clone(),
            lot_sizing_rule: LotSizingRule::LotForLot,
            minimum_order_qty: None,
            maximum_order_qty: None,
            order_multiple: None,
            safety_stock: Decimal::ZERO,
        })
        .collect()
}
```

### 處理虛設零件

```rust
// 虛設件立即消耗，不單獨規劃
fn handle_phantom_components(
    bom_graph: &BomGraph,
    explosion: &ExplosionResult,
) -> Vec<Demand> {
    explosion
        .items
        .iter()
        .filter(|item| {
            // 在 MRP 規劃中跳過虛設零件
            let component = bom_graph.get_component(&item.component_id).unwrap();
            !matches!(component.component_type, ComponentType::Phantom)
        })
        .map(|item| Demand {
            item_id: item.component_id.to_string(),
            quantity: item.total_quantity,
            due_date: item.required_date,
            demand_type: DemandType::DependentDemand,
            source_id: None,
        })
        .collect()
}
```

### 增量更新

```rust
use mrp_cache::IncrementalCache;

// 使用快取進行高效重新規劃
fn incremental_replanning(
    bom_graph: &BomGraph,
    mrp_cache: &mut IncrementalCache,
    changed_demands: Vec<Demand>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 只重新計算受影響的項目
    let affected_items = mrp_cache.get_affected_items(&changed_demands);

    // 只重新展開變更的頂層項目
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    for demand in changed_demands {
        let explosion = explosion_calc.explode(
            &ComponentId::new(&demand.item_id),
            demand.quantity,
        )?;

        // 使用新展開結果更新快取
        mrp_cache.update_explosion(&demand.item_id, explosion);
    }

    // 只對受影響項目重新計算 MRP
    let mrp_result = mrp_cache.calculate_incremental(&affected_items)?;

    Ok(())
}
```

## 最佳實踐

### 1. 快取 BOM 展開

```rust
use std::collections::HashMap;

struct BomCache {
    explosions: HashMap<String, ExplosionResult>,
}

impl BomCache {
    fn get_or_calculate(&mut self,
        bom_graph: &BomGraph,
        item_id: &str,
        quantity: Decimal
    ) -> &ExplosionResult {
        self.explosions.entry(item_id.to_string()).or_insert_with(|| {
            ExplosionCalculator::new(bom_graph)
                .explode(&ComponentId::new(item_id), quantity)
                .unwrap()
        })
    }
}
```

### 2. 驗證資料一致性

```rust
fn validate_bom_mrp_consistency(
    bom_graph: &BomGraph,
    mrp_configs: &[MrpConfig],
) -> Result<(), String> {
    // 確保所有 BOM 零件都有 MRP 配置
    for component in bom_graph.get_all_components() {
        let has_config = mrp_configs
            .iter()
            .any(|cfg| cfg.item_id == component.id.to_string());

        if !has_config {
            return Err(format!(
                "BOM 中的零件 {} 沒有 MRP 配置",
                component.id
            ));
        }
    }

    Ok(())
}
```

### 3. 處理提前期偏移

```rust
// 考慮 BOM 階層計算訂單日期
fn calculate_order_dates_with_bom_levels(
    bom_graph: &BomGraph,
    top_level_due_date: NaiveDate,
) -> HashMap<String, NaiveDate> {
    let mut order_dates = HashMap::new();

    // 反向遍歷 BOM（由下而上）
    for level in bom_graph.get_levels_bottom_up() {
        for component in level {
            let lead_time = component.lead_time_days;
            let parent_dates: Vec<NaiveDate> = bom_graph
                .get_parents(&component.id)
                .iter()
                .map(|parent_id| {
                    *order_dates.get(&parent_id.to_string())
                        .unwrap_or(&top_level_due_date)
                })
                .collect();

            let earliest_parent_date = parent_dates.iter().min()
                .unwrap_or(&top_level_due_date);

            let order_date = *earliest_parent_date - chrono::Duration::days(lead_time as i64);
            order_dates.insert(component.id.to_string(), order_date);
        }
    }

    order_dates
}
```

### 4. 監控效能

```rust
use std::time::Instant;

fn benchmark_integrated_system() {
    let start = Instant::now();

    // BOM 展開
    let explosion_start = Instant::now();
    let explosion = explode_bom();
    println!("BOM 展開: {:?}", explosion_start.elapsed());

    // MRP 計算
    let mrp_start = Instant::now();
    let mrp_result = calculate_mrp();
    println!("MRP 計算: {:?}", mrp_start.elapsed());

    println!("總時間: {:?}", start.elapsed());
}
```

## 疑難排解

### 問題：BOM 循環相依

**問題**：由於 BOM 中的循環引用導致 MRP 計算失敗

**解決方案**：
```rust
// 使用 BOM 圖驗證
if let Err(e) = bom_graph.validate_no_cycles() {
    eprintln!("BOM 包含循環相依: {}", e);
    // 適當處理錯誤
}
```

### 問題：提前期不匹配

**問題**：MRP 訂單計算過晚

**解決方案**：
```rust
// 始終從 BOM 同步提前期到 MRP 配置
for component in bom_graph.get_all_components() {
    let mrp_config = mrp_configs.iter_mut()
        .find(|cfg| cfg.item_id == component.id.to_string())
        .unwrap();

    mrp_config.lead_time_days = component.lead_time_days;
}
```

### 問題：大型 BOM 的記憶體使用

**問題**：複雜產品結構導致高記憶體消耗

**解決方案**：
```rust
// 使用串流展開而非完全具體化
let explosion_stream = ExplosionCalculator::new(&bom_graph)
    .explode_streaming(&root_id, quantity);

for batch in explosion_stream.chunks(1000) {
    process_demands_batch(batch);
}
```

## 相關文件

- [NexusBom 文件](https://github.com/Ricemug/NexusBom)
- [NexusMRP 文件](../README.md)
- [動態時間桶](./DYNAMIC_TIME_BUCKETS.md)
- [負庫存處理](./NEGATIVE_INVENTORY.md)

## 支援

如有整合問題：
- 在 [NexusMRP GitHub](https://github.com/Ricemug/NexusMRP/issues) 建立 issue
- Email：xiaoivan1@proton.me

---

**祝您規劃順利！🚀**
