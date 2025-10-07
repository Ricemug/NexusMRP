# 🔗 集成指南：NexusMRP + NexusBom

> 物料需求计划与物料清单集成完整指南

[English](./INTEGRATION_WITH_NEXUSBOM.md) | [繁體中文](./INTEGRATION_WITH_NEXUSBOM.zh-TW.md)

本指南说明如何集成 **NexusMRP**（物料需求计划）与 **NexusBom**（物料清单）以构建完整的制造规划系统。

## 📋 目录

- [概述](#概述)
- [为什么要集成？](#为什么要集成)
- [架构](#架构)
- [集成步骤](#集成步骤)
- [代码示例](#代码示例)
- [最佳实践](#最佳实践)
- [疑难解答](#疑难解答)

## 概述

**NexusBom** 和 **NexusMRP** 被设计为互补系统：

- **NexusBom**：管理产品结构、物料展开和成本计算
- **NexusMRP**：规划物料需求、排程生产和管理库存

两者结合形成强大的制造规划解决方案。

## 为什么要集成？

| 未集成 | 已集成 |
|--------|--------|
| 手动 BOM 查询 | 自动物料展开 |
| 静态规划 | 动态需求传播 |
| 系统分离 | 端到端可视化 |
| 有限优化 | 产能感知规划 |

### 主要优势

✅ **自动多阶规划** - MRP 使用 BOM 将需求展开至所有层级
✅ **实时成本分析** - 结合计划订单与 BOM 成本
✅ **变更影响分析** - 查看 BOM 变更如何影响物料计划
✅ **虚设件处理** - MRP 遵循 BOM 虚设零件规则
✅ **替代 BOM 支持** - 以不同制造路径进行规划

## 架构

```
┌─────────────────────────────────────────────────────────┐
│                   您的应用程序                           │
└─────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┴───────────────┐
           │                               │
           ▼                               ▼
┌──────────────────────┐       ┌──────────────────────┐
│     NexusBom         │       │     NexusMRP         │
│   (BOM 结构)         │◄──────│   (规划逻辑)         │
└──────────────────────┘       └──────────────────────┘
           │                               │
           │     物料展开                   │
           │     零件清单                   │
           │     成本数据                   │
           └───────────────────────────────┘
```

### 数据流程

1. **加载 BOM 数据** → NexusBom 建立产品结构图
2. **创建需求** → NexusMRP 接收顶层需求
3. **展开 BOM** → NexusBom 提供零件清单与数量
4. **计算 MRP** → NexusMRP 通过 BOM 层级传播需求
5. **生成计划** → 输出所有零件的计划订单

## 集成步骤

### 步骤 1：添加依赖包

在 `Cargo.toml` 中添加两个库：

```toml
[dependencies]
# NexusBom - BOM 计算引擎
bom-core = { git = "https://github.com/Ricemug/NexusBom" }
bom-calc = { git = "https://github.com/Ricemug/NexusBom" }
bom-graph = { git = "https://github.com/Ricemug/NexusBom" }

# NexusMRP - MRP 计算引擎
mrp-core = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-calc = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-cache = { git = "https://github.com/Ricemug/NexusMRP" }
```

### 步骤 2：建立 BOM 图

```rust
use bom_core::*;
use bom_graph::BomGraph;

// 定义产品结构
let components = vec![
    Component {
        id: ComponentId::new("BIKE-001"),
        description: "完整自行车".to_string(),
        component_type: ComponentType::FinishedProduct,
        standard_cost: Some(Decimal::new(50000, 2)), // $500
        lead_time_days: 5,
        procurement_type: ProcurementType::Make,
    },
    Component {
        id: ComponentId::new("FRAME-001"),
        description: "自行车车架".to_string(),
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
    // ... 更多 BOM 关系
];

// 建立 BOM 图
let bom_graph = BomGraph::from_components(&components, &bom_items)?;
```

### 步骤 3：执行物料展开

```rust
use bom_calc::ExplosionCalculator;

// 针对特定数量展开 BOM
let explosion_calc = ExplosionCalculator::new(&bom_graph);
let explosion_result = explosion_calc.explode(
    &ComponentId::new("BIKE-001"),
    Decimal::from(100), // 数量：100 台自行车
)?;

// 获取扁平化的零件需求
let component_requirements = explosion_result.get_flattened_requirements();
```

### 步骤 4：从 BOM 创建 MRP 需求

```rust
use mrp_core::*;
use chrono::NaiveDate;

// 将 BOM 展开转换为 MRP 需求
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

### 步骤 5：使用 BOM 提前期配置 MRP

```rust
use mrp_calc::MRPCalculator;

// 使用 BOM 数据创建 MRP 配置
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

### 步骤 6：执行集成 MRP 计算

```rust
// 初始化 MRP 计算器
let mrp_calculator = MRPCalculator::new(mrp_configs);

// 使用基于 BOM 的需求执行 MRP
let mrp_result = mrp_calculator.calculate(
    &demands,
    &existing_supplies,    // 任何现有采购单或生产单
    &inventory_balances,   // 当前库存
)?;

// 获取计划订单
let planned_orders = mrp_result.planned_orders;

println!("生成 {} 笔计划订单", planned_orders.len());
for order in planned_orders {
    println!("  {} - 数量: {} - 日期: {}",
        order.item_id, order.quantity, order.due_date);
}
```

## 代码示例

### 完整集成示例

```rust
use bom_core::*;
use bom_graph::BomGraph;
use bom_calc::ExplosionCalculator;
use mrp_core::*;
use mrp_calc::MRPCalculator;
use rust_decimal::Decimal;
use chrono::NaiveDate;

fn integrated_planning_example() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 建立 BOM 结构
    let bom_graph = build_bicycle_bom()?;

    // 2. 接收客户订单
    let customer_order = CustomerOrder {
        product_id: "BIKE-001".to_string(),
        quantity: Decimal::from(100),
        due_date: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
    };

    // 3. 展开 BOM 以获取零件需求
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    let explosion = explosion_calc.explode_with_lead_time_offset(
        &ComponentId::new(&customer_order.product_id),
        customer_order.quantity,
        customer_order.due_date,
    )?;

    // 4. 转换为 MRP 需求
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

    // 5. 执行 MRP 计算
    let mrp_configs = extract_mrp_configs_from_bom(&bom_graph);
    let calculator = MRPCalculator::new(mrp_configs);

    let mrp_result = calculator.calculate(
        &demands,
        &vec![], // 无现有供应
        &vec![], // 无现有库存
    )?;

    // 6. 输出计划订单
    println!("客户订单 {} 的计划订单：", customer_order.product_id);
    for order in mrp_result.planned_orders {
        println!("  订单: {} - 数量: {} - 开始: {} - 到期: {}",
            order.item_id,
            order.quantity,
            order.order_date,
            order.due_date
        );
    }

    // 7. 使用 BOM 计算总成本
    let total_cost = calculate_order_cost(&bom_graph, &mrp_result.planned_orders)?;
    println!("总物料成本: ${:.2}", total_cost);

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

### 处理虚设零件

```rust
// 虚设件立即消耗，不单独规划
fn handle_phantom_components(
    bom_graph: &BomGraph,
    explosion: &ExplosionResult,
) -> Vec<Demand> {
    explosion
        .items
        .iter()
        .filter(|item| {
            // 在 MRP 规划中跳过虚设零件
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

// 使用缓存进行高效重新规划
fn incremental_replanning(
    bom_graph: &BomGraph,
    mrp_cache: &mut IncrementalCache,
    changed_demands: Vec<Demand>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 只重新计算受影响的项目
    let affected_items = mrp_cache.get_affected_items(&changed_demands);

    // 只重新展开变更的顶层项目
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    for demand in changed_demands {
        let explosion = explosion_calc.explode(
            &ComponentId::new(&demand.item_id),
            demand.quantity,
        )?;

        // 使用新展开结果更新缓存
        mrp_cache.update_explosion(&demand.item_id, explosion);
    }

    // 只对受影响项目重新计算 MRP
    let mrp_result = mrp_cache.calculate_incremental(&affected_items)?;

    Ok(())
}
```

## 最佳实践

### 1. 缓存 BOM 展开

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

### 2. 验证数据一致性

```rust
fn validate_bom_mrp_consistency(
    bom_graph: &BomGraph,
    mrp_configs: &[MrpConfig],
) -> Result<(), String> {
    // 确保所有 BOM 零件都有 MRP 配置
    for component in bom_graph.get_all_components() {
        let has_config = mrp_configs
            .iter()
            .any(|cfg| cfg.item_id == component.id.to_string());

        if !has_config {
            return Err(format!(
                "BOM 中的零件 {} 没有 MRP 配置",
                component.id
            ));
        }
    }

    Ok(())
}
```

### 3. 处理提前期偏移

```rust
// 考虑 BOM 层级计算订单日期
fn calculate_order_dates_with_bom_levels(
    bom_graph: &BomGraph,
    top_level_due_date: NaiveDate,
) -> HashMap<String, NaiveDate> {
    let mut order_dates = HashMap::new();

    // 反向遍历 BOM（由下而上）
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

### 4. 监控性能

```rust
use std::time::Instant;

fn benchmark_integrated_system() {
    let start = Instant::now();

    // BOM 展开
    let explosion_start = Instant::now();
    let explosion = explode_bom();
    println!("BOM 展开: {:?}", explosion_start.elapsed());

    // MRP 计算
    let mrp_start = Instant::now();
    let mrp_result = calculate_mrp();
    println!("MRP 计算: {:?}", mrp_start.elapsed());

    println!("总时间: {:?}", start.elapsed());
}
```

## 疑难解答

### 问题：BOM 循环依赖

**问题**：由于 BOM 中的循环引用导致 MRP 计算失败

**解决方案**：
```rust
// 使用 BOM 图验证
if let Err(e) = bom_graph.validate_no_cycles() {
    eprintln!("BOM 包含循环依赖: {}", e);
    // 适当处理错误
}
```

### 问题：提前期不匹配

**问题**：MRP 订单计算过晚

**解决方案**：
```rust
// 始终从 BOM 同步提前期到 MRP 配置
for component in bom_graph.get_all_components() {
    let mrp_config = mrp_configs.iter_mut()
        .find(|cfg| cfg.item_id == component.id.to_string())
        .unwrap();

    mrp_config.lead_time_days = component.lead_time_days;
}
```

### 问题：大型 BOM 的内存使用

**问题**：复杂产品结构导致高内存消耗

**解决方案**：
```rust
// 使用流式展开而非完全具体化
let explosion_stream = ExplosionCalculator::new(&bom_graph)
    .explode_streaming(&root_id, quantity);

for batch in explosion_stream.chunks(1000) {
    process_demands_batch(batch);
}
```

## 相关文档

- [NexusBom 文档](https://github.com/Ricemug/NexusBom)
- [NexusMRP 文档](../README.md)
- [动态时间桶](./DYNAMIC_TIME_BUCKETS.md)
- [负库存处理](./NEGATIVE_INVENTORY.md)

## 支持

如有集成问题：
- 在 [NexusMRP GitHub](https://github.com/Ricemug/NexusMRP/issues) 创建 issue
- Email：xiaoivan1@proton.me

---

**祝您规划顺利！🚀**
