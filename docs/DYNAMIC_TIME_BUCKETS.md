# 動態時間桶 (Dynamic Time Buckets)

## 概述

動態時間桶功能確保 MRP 計算能正確處理 BOM 展開後產生的相依需求日期。

## 問題背景

### 原始問題

在多階 BOM 計算中：

1. **初始時間桶創建**：系統在計算開始時，只基於**獨立需求**和**初始供應**創建時間桶
2. **BOM 展開**：父件的計劃訂單產生子件的相依需求
3. **日期遺漏**：相依需求的日期可能**不在原始時間桶中**
4. **計算錯誤**：淨需求計算會遺漏這些新日期的需求

### 範例場景

```
父件 A（自行車）
├─ 獨立需求：11/10, 數量 100
├─ 提前期：7 天
└─ 計劃訂單：11/3（開始生產日）

子件 B（車架）
└─ 相依需求：11/3, 數量 100  ← 新日期，不在原始時間桶中！
```

**原始時間桶**: `[11/10]` （只有父件的需求日期）

**問題**: 子件 B 的相依需求在 11/3，但時間桶中沒有這個日期，導致計算錯誤。

## 解決方案

### 實作方式

為每個物料**動態創建專屬時間桶**，合併：
1. 基礎時間桶（來自獨立需求）
2. 該物料的所有實際需求日期（獨立 + 相依）
3. 該物料的所有供應日期

### 核心邏輯

**位置**: `crates/mrp-calc/src/calculator.rs`

```rust
/// 為單一物料創建動態時間桶
fn create_component_time_buckets(
    &self,
    base_time_buckets: &[chrono::NaiveDate],
    component_demands: &[Demand],
    component_supplies: &[Supply],
) -> Vec<chrono::NaiveDate> {
    use std::collections::HashSet;

    // 使用 HashSet 收集所有唯一日期
    let mut dates: HashSet<chrono::NaiveDate> =
        base_time_buckets.iter().copied().collect();

    // 添加該物料所有需求日期
    for demand in component_demands {
        dates.insert(demand.required_date);
    }

    // 添加該物料所有供應日期
    for supply in component_supplies {
        dates.insert(supply.available_date);
    }

    // 轉換為 Vec 並排序
    let mut sorted_dates: Vec<chrono::NaiveDate> = dates.into_iter().collect();
    sorted_dates.sort();

    sorted_dates
}
```

### 計算流程

```rust
// Step 1: 創建基礎時間桶（來自獨立需求）
let time_buckets = BucketingCalculator::create_time_buckets(
    &demands,
    &supplies,
    planning_horizon,
);

// Step 2: 逐物料計算（迭代處理 BOM 展開）
while !components_to_process.is_empty() {
    let component_id = components_to_process.remove(0);

    // 合併獨立需求和相依需求
    let mut component_demands = grouped_demands.get(&component_id)...;
    component_demands.extend(dependent_demands.get(&component_id));

    // ✅ 動態創建時間桶（包含所有實際需求日期）
    let component_time_buckets = self.create_component_time_buckets(
        &time_buckets,
        &component_demands,
        &component_supplies,
    );

    // 使用動態時間桶計算淨需求
    let net_requirements = NettingCalculator::calculate(
        &component_demands,
        &component_supplies,
        initial_inventory,
        config.safety_stock,
        &component_time_buckets,  // ← 使用動態時間桶
        config.allow_negative_inventory,
    )?;
}
```

## 範例演示

### 場景：兩階 BOM

```
產品 BIKE-001 (自行車)
└─ 子件 FRAME-001 (車架) x 1

需求：
- BIKE-001: 11/10, 數量 100 (銷售訂單)

配置：
- BIKE-001: 提前期 7 天
- FRAME-001: 提前期 5 天
```

### 計算過程

#### 階段 1: 計算父件 BIKE-001

**時間桶**: `[11/10]`

**淨需求**:
```
日期     | 總需求 | 預計收貨 | 預計庫存 | 淨需求
---------|--------|----------|----------|--------
11/10    | 100    | 0        | -100     | 100
```

**計劃訂單**:
```
訂單日期 | 需求日期 | 數量
---------|----------|------
11/3     | 11/10    | 100   ← 考慮 7 天提前期
```

#### 階段 2: BOM 展開

父件計劃訂單 → 產生子件相依需求

**FRAME-001 相依需求**:
- 日期: `11/3` (父件生產開始日)
- 數量: `100 × 1 = 100`

#### 階段 3: 計算子件 FRAME-001

**原始時間桶**: `[11/10]`

**動態時間桶**: `[11/3, 11/10]` ✅ 包含相依需求日期

**淨需求**:
```
日期     | 總需求 | 預計收貨 | 預計庫存 | 淨需求
---------|--------|----------|----------|--------
11/3     | 100    | 0        | -100     | 100   ← 正確計算
11/10    | 0      | 0        | -100     | 0
```

**計劃訂單**:
```
訂單日期 | 需求日期 | 數量
---------|----------|------
10/29    | 11/3     | 100   ← 考慮 5 天提前期
```

### 沒有動態時間桶的錯誤情況

**時間桶**: `[11/10]` (遺漏 11/3)

**淨需求** (錯誤):
```
日期     | 總需求 | 預計收貨 | 預計庫存 | 淨需求
---------|--------|----------|----------|--------
11/10    | 100    | 0        | -100     | 100   ← 日期錯誤！
                                                 (需求實際在 11/3)
```

**結果**: 計劃訂單日期錯誤，無法滿足父件生產需求。

## 技術細節

### 性能考量

1. **HashSet 去重**: 自動處理重複日期，避免冗餘計算
2. **按需創建**: 只為實際計算的物料創建動態時間桶
3. **排序一次**: 創建後立即排序，後續使用時無需再排序

### 時間複雜度

- **創建動態時間桶**: `O(n log n)`
  - `n` = 基礎時間桶 + 需求數 + 供應數
  - 主要開銷在排序
- **空間複雜度**: `O(n)`

### 與其他功能的整合

| 功能 | 影響 |
|-----|------|
| **淨需求計算** | 使用動態時間桶，確保所有日期都被計算 |
| **批量規則** | 基於完整的淨需求結果，不受影響 |
| **提前期計算** | 工作日曆正常運作，不受影響 |
| **需求追溯 (Pegging)** | 可正確追溯相依需求，不受影響 |

## 測試驗證

### 單元測試

**位置**: `crates/mrp-calc/src/calculator.rs::tests::test_dynamic_time_buckets`

```rust
#[test]
fn test_dynamic_time_buckets() {
    // 基礎時間桶
    let base_buckets = vec![
        NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
    ];

    // 相依需求（新日期）
    let component_demands = vec![
        Demand::new(..., 2025-11-03, ...),  // 新日期
        Demand::new(..., 2025-11-08, ...),  // 新日期
    ];

    // 供應（新日期）
    let component_supplies = vec![
        Supply::new(..., 2025-11-07, ...),  // 新日期
    ];

    // 創建動態時間桶
    let dynamic_buckets = calculator.create_component_time_buckets(
        &base_buckets,
        &component_demands,
        &component_supplies,
    );

    // 驗證：包含所有日期，正確排序，無重複
    assert_eq!(dynamic_buckets, vec![
        2025-11-01,
        2025-11-03,  // ✅ 相依需求日期
        2025-11-05,
        2025-11-07,  // ✅ 供應日期
        2025-11-08,  // ✅ 相依需求日期
    ]);
}
```

### 集成測試

使用完整的 BOM 展開流程驗證：
- 多階 BOM 計算
- 相依需求正確產生
- 計劃訂單日期正確

## 日誌輸出

啟用 tracing 日誌後，可看到動態時間桶的創建過程：

```
DEBUG mrp_calc::calculator: 計算物料 MRP: FRAME-001
DEBUG mrp_calc::calculator: 物料 FRAME-001 時間桶: 基礎 2 個, 擴展後 5 個
```

## 版本歷史

- **v0.1.0**: 初始實現
  - 添加 `create_component_time_buckets` 方法
  - 修改 `calculate_component_mrp` 使用動態時間桶
  - 添加單元測試 `test_dynamic_time_buckets`
  - 完整測試覆蓋（43 tests ✓）

## 參考資料

- [MRP 計算邏輯](./MRP_CALCULATION.md)
- [時間分桶策略](../crates/mrp-calc/src/bucketing.rs)
- [BOM 展開邏輯](./BOM_EXPLOSION.md)
