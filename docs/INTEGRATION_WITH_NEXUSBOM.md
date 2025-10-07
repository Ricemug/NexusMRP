# 🔗 Integration Guide: NexusMRP + NexusBom

This guide explains how to integrate **NexusMRP** (Material Requirements Planning) with **NexusBom** (Bill of Materials) to build a complete manufacturing planning system.

## 📋 Table of Contents

- [Overview](#overview)
- [Why Integrate?](#why-integrate)
- [Architecture](#architecture)
- [Integration Steps](#integration-steps)
- [Code Examples](#code-examples)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

**NexusBom** and **NexusMRP** are designed as complementary systems:

- **NexusBom**: Manages product structures, material explosions, and cost calculations
- **NexusMRP**: Plans material requirements, schedules production, and manages inventory

Together, they form a powerful manufacturing planning solution.

## Why Integrate?

| Without Integration | With Integration |
|---------------------|------------------|
| Manual BOM lookups | Automatic material explosion |
| Static planning | Dynamic demand propagation |
| Disconnected systems | End-to-end visibility |
| Limited optimization | Capacity-aware planning |

### Key Benefits

✅ **Automatic Multi-Level Planning** - MRP uses BOM to explode demands through all levels
✅ **Real-Time Cost Analysis** - Combine planned orders with BOM costs
✅ **Change Impact Analysis** - See how BOM changes affect material plans
✅ **Phantom Part Handling** - MRP respects BOM phantom components
✅ **Alternate BOM Support** - Plan with different manufacturing routes

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Your Application                       │
└─────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┴───────────────┐
           │                               │
           ▼                               ▼
┌──────────────────────┐       ┌──────────────────────┐
│     NexusBom         │       │     NexusMRP         │
│  (BOM Structure)     │◄──────│  (Planning Logic)    │
└──────────────────────┘       └──────────────────────┘
           │                               │
           │     Material Explosion        │
           │     Component Lists           │
           │     Cost Data                 │
           └───────────────────────────────┘
```

### Data Flow

1. **Load BOM Data** → NexusBom builds product structure graph
2. **Create Demands** → NexusMRP receives top-level requirements
3. **Explode BOM** → NexusBom provides component lists with quantities
4. **Calculate MRP** → NexusMRP propagates demands through BOM levels
5. **Generate Plans** → Output planned orders for all components

## Integration Steps

### Step 1: Add Dependencies

Add both libraries to your `Cargo.toml`:

```toml
[dependencies]
# NexusBom - BOM calculation engine
bom-core = { git = "https://github.com/Ricemug/NexusBom" }
bom-calc = { git = "https://github.com/Ricemug/NexusBom" }
bom-graph = { git = "https://github.com/Ricemug/NexusBom" }

# NexusMRP - MRP calculation engine
mrp-core = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-calc = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-cache = { git = "https://github.com/Ricemug/NexusMRP" }
```

### Step 2: Build BOM Graph

```rust
use bom_core::*;
use bom_graph::BomGraph;

// Define your product structure
let components = vec![
    Component {
        id: ComponentId::new("BIKE-001"),
        description: "Complete Bicycle".to_string(),
        component_type: ComponentType::FinishedProduct,
        standard_cost: Some(Decimal::new(50000, 2)), // $500
        lead_time_days: 5,
        procurement_type: ProcurementType::Make,
    },
    Component {
        id: ComponentId::new("FRAME-001"),
        description: "Bike Frame".to_string(),
        component_type: ComponentType::SubAssembly,
        standard_cost: Some(Decimal::new(20000, 2)), // $200
        lead_time_days: 10,
        procurement_type: ProcurementType::Buy,
    },
    // ... more components
];

let bom_items = vec![
    BomItem {
        parent_id: ComponentId::new("BIKE-001"),
        child_id: ComponentId::new("FRAME-001"),
        quantity: Decimal::ONE,
        sequence: 10,
        is_phantom: false,
    },
    // ... more BOM relationships
];

// Build the BOM graph
let bom_graph = BomGraph::from_components(&components, &bom_items)?;
```

### Step 3: Perform Material Explosion

```rust
use bom_calc::ExplosionCalculator;

// Explode BOM for a specific quantity
let explosion_calc = ExplosionCalculator::new(&bom_graph);
let explosion_result = explosion_calc.explode(
    &ComponentId::new("BIKE-001"),
    Decimal::from(100), // Quantity: 100 bikes
)?;

// Get flattened component requirements
let component_requirements = explosion_result.get_flattened_requirements();
```

### Step 4: Create MRP Demands from BOM

```rust
use mrp_core::*;
use chrono::NaiveDate;

// Convert BOM explosion to MRP demands
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

### Step 5: Configure MRP with BOM Lead Times

```rust
use mrp_calc::MRPCalculator;

// Create MRP configurations using BOM data
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

### Step 6: Run Integrated MRP Calculation

```rust
// Initialize MRP calculator
let mrp_calculator = MRPCalculator::new(mrp_configs);

// Run MRP with BOM-based demands
let mrp_result = mrp_calculator.calculate(
    &demands,
    &existing_supplies,    // Any existing POs or production orders
    &inventory_balances,   // Current inventory
)?;

// Get planned orders
let planned_orders = mrp_result.planned_orders;

println!("Generated {} planned orders", planned_orders.len());
for order in planned_orders {
    println!("  {} - Qty: {} - Date: {}",
        order.item_id, order.quantity, order.due_date);
}
```

## Code Examples

### Complete Integration Example

```rust
use bom_core::*;
use bom_graph::BomGraph;
use bom_calc::ExplosionCalculator;
use mrp_core::*;
use mrp_calc::MRPCalculator;
use rust_decimal::Decimal;
use chrono::NaiveDate;

fn integrated_planning_example() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build BOM structure
    let bom_graph = build_bicycle_bom()?;

    // 2. Receive customer order
    let customer_order = CustomerOrder {
        product_id: "BIKE-001".to_string(),
        quantity: Decimal::from(100),
        due_date: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
    };

    // 3. Explode BOM to get component requirements
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    let explosion = explosion_calc.explode_with_lead_time_offset(
        &ComponentId::new(&customer_order.product_id),
        customer_order.quantity,
        customer_order.due_date,
    )?;

    // 4. Convert to MRP demands
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

    // 5. Run MRP calculation
    let mrp_configs = extract_mrp_configs_from_bom(&bom_graph);
    let calculator = MRPCalculator::new(mrp_configs);

    let mrp_result = calculator.calculate(
        &demands,
        &vec![], // No existing supplies
        &vec![], // No existing inventory
    )?;

    // 6. Output planned orders
    println!("Planned Orders for Customer Order {}:", customer_order.product_id);
    for order in mrp_result.planned_orders {
        println!("  Order: {} - Qty: {} - Start: {} - Due: {}",
            order.item_id,
            order.quantity,
            order.order_date,
            order.due_date
        );
    }

    // 7. Calculate total cost using BOM
    let total_cost = calculate_order_cost(&bom_graph, &mrp_result.planned_orders)?;
    println!("Total Material Cost: ${:.2}", total_cost);

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

### Handling Phantom Components

```rust
// Phantom parts are consumed immediately, not planned separately
fn handle_phantom_components(
    bom_graph: &BomGraph,
    explosion: &ExplosionResult,
) -> Vec<Demand> {
    explosion
        .items
        .iter()
        .filter(|item| {
            // Skip phantom components in MRP planning
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

### Incremental Updates

```rust
use mrp_cache::IncrementalCache;

// Use caching for efficient replanning
fn incremental_replanning(
    bom_graph: &BomGraph,
    mrp_cache: &mut IncrementalCache,
    changed_demands: Vec<Demand>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Only recalculate affected items
    let affected_items = mrp_cache.get_affected_items(&changed_demands);

    // Re-explode only changed top-level items
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    for demand in changed_demands {
        let explosion = explosion_calc.explode(
            &ComponentId::new(&demand.item_id),
            demand.quantity,
        )?;

        // Update cache with new explosions
        mrp_cache.update_explosion(&demand.item_id, explosion);
    }

    // Recalculate MRP for affected items only
    let mrp_result = mrp_cache.calculate_incremental(&affected_items)?;

    Ok(())
}
```

## Best Practices

### 1. Cache BOM Explosions

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

### 2. Validate Data Consistency

```rust
fn validate_bom_mrp_consistency(
    bom_graph: &BomGraph,
    mrp_configs: &[MrpConfig],
) -> Result<(), String> {
    // Ensure all BOM components have MRP configs
    for component in bom_graph.get_all_components() {
        let has_config = mrp_configs
            .iter()
            .any(|cfg| cfg.item_id == component.id.to_string());

        if !has_config {
            return Err(format!(
                "Component {} in BOM has no MRP configuration",
                component.id
            ));
        }
    }

    Ok(())
}
```

### 3. Handle Lead Time Offsets

```rust
// Calculate order dates considering BOM levels
fn calculate_order_dates_with_bom_levels(
    bom_graph: &BomGraph,
    top_level_due_date: NaiveDate,
) -> HashMap<String, NaiveDate> {
    let mut order_dates = HashMap::new();

    // Traverse BOM in reverse (bottom-up)
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

### 4. Monitor Performance

```rust
use std::time::Instant;

fn benchmark_integrated_system() {
    let start = Instant::now();

    // BOM explosion
    let explosion_start = Instant::now();
    let explosion = explode_bom();
    println!("BOM Explosion: {:?}", explosion_start.elapsed());

    // MRP calculation
    let mrp_start = Instant::now();
    let mrp_result = calculate_mrp();
    println!("MRP Calculation: {:?}", mrp_start.elapsed());

    println!("Total Time: {:?}", start.elapsed());
}
```

## Troubleshooting

### Issue: Circular BOM Dependencies

**Problem**: MRP calculation fails due to circular references in BOM

**Solution**:
```rust
// Use BOM graph validation
if let Err(e) = bom_graph.validate_no_cycles() {
    eprintln!("BOM contains circular dependencies: {}", e);
    // Handle error appropriately
}
```

### Issue: Mismatched Lead Times

**Problem**: MRP orders calculated too late

**Solution**:
```rust
// Always sync lead times from BOM to MRP configs
for component in bom_graph.get_all_components() {
    let mrp_config = mrp_configs.iter_mut()
        .find(|cfg| cfg.item_id == component.id.to_string())
        .unwrap();

    mrp_config.lead_time_days = component.lead_time_days;
}
```

### Issue: Memory Usage with Large BOMs

**Problem**: High memory consumption with complex product structures

**Solution**:
```rust
// Use streaming explosion instead of full materialization
let explosion_stream = ExplosionCalculator::new(&bom_graph)
    .explode_streaming(&root_id, quantity);

for batch in explosion_stream.chunks(1000) {
    process_demands_batch(batch);
}
```

## Related Documentation

- [NexusBom Documentation](https://github.com/Ricemug/NexusBom)
- [NexusMRP Documentation](../README.md)
- [Dynamic Time Buckets](./DYNAMIC_TIME_BUCKETS.md)
- [Negative Inventory Handling](./NEGATIVE_INVENTORY.md)

## Support

For integration questions:
- Create an issue on [NexusMRP GitHub](https://github.com/Ricemug/NexusMRP/issues)
- Email: xiaoivan1@proton.me

---

**Happy Planning! 🚀**
