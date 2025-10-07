# 負庫存控制參數 (Allow Negative Inventory)

## 概述

`allow_negative_inventory` 參數控制 MRP 系統對負庫存的容忍度，影響計劃訂單的產生時機。

## 技術說明

### 參數位置

- **Rust**: `mrp_core::MrpConfig::allow_negative_inventory: bool`
- **Python**: `mrp_engine.MrpConfig.allow_negative_inventory: bool`
- **預設值**: `false` （不允許負庫存）

### 計算邏輯

#### 不允許負庫存 (`allow_negative_inventory = false`)

```rust
if projected_on_hand < safety_stock {
    net_requirement = safety_stock - projected_on_hand
} else {
    net_requirement = 0
}
```

**行為**：
- 當預計庫存低於安全庫存時，立即產生淨需求
- 計劃訂單補足到安全庫存水位以上
- 保守策略，確保庫存充足

**範例**：
```
初始庫存: 30
安全庫存: 10
需求: 100

預計庫存 = 30 - 100 = -70
淨需求 = 10 - (-70) = 80  ← 補足到安全庫存
```

#### 允許負庫存 (`allow_negative_inventory = true`)

```rust
if projected_on_hand < 0 {
    net_requirement = -projected_on_hand  // 補足到 0
} else {
    net_requirement = 0  // 即使低於安全庫存也不產生需求
}
```

**行為**：
- 忽略安全庫存
- 只有當預計庫存為負時才產生淨需求
- 計劃訂單僅補足到 0
- 激進策略，延遲補貨

**範例**：
```
初始庫存: 30
安全庫存: 10 (被忽略)
需求: 100

預計庫存 = 30 - 100 = -70
淨需求 = 70  ← 只補足到 0，忽略安全庫存
```

## 使用場景

### ❌ 不允許負庫存 (False) - 預設

| 生產模式 | 物料類型 | 說明 |
|---------|---------|------|
| **MTS** (Make-to-Stock) | 成品、半成品 | 批量生產，需維持庫存 |
| **庫存管理** | 原物料、包材 | 有實體倉儲，需安全庫存 |
| **連續生產** | 大宗商品 | 不能斷料，必須保持庫存 |

**ERP 對應**：
- SAP: MRP Type = `PD` (MRP)
- Oracle: Planning Method = `MRP Planning`
- Dynamics: Item Coverage = `Requirement`

### ✅ 允許負庫存 (True)

| 生產模式 | 物料類型 | 說明 |
|---------|---------|------|
| **MTO** (Make-to-Order) | 客製化產品 | 接單後生產，無現貨 |
| **PTO** (Purchase-to-Order) | 特殊採購件 | 接單後才採購 |
| **虛擬件** | Phantom/虛設件 | BOM 結構用，不實際生產 |
| **服務** | 工時、服務 | 無實體庫存概念 |
| **工程專案** | 專案物料 | 一次性，不需庫存 |

**ERP 對應**：
- SAP: MRP Type = `ND` (No Planning) 或 `VB` (Manual Reorder)
- Oracle: Planning Method = `MPS/MTO`
- Dynamics: Item Coverage = `Manual`

## 從 Python/ERP 設置

### 基本用法

```python
from mrp_engine import MrpConfig

# MTS 模式 - 不允許負庫存
config_mts = MrpConfig(
    component_id="PRODUCT-001",
    lead_time_days=5,
    procurement_type="Make",
    allow_negative_inventory=False  # 預設值
)
config_mts.safety_stock = 50.0

# MTO 模式 - 允許負庫存
config_mto = MrpConfig(
    component_id="CUSTOM-001",
    lead_time_days=10,
    procurement_type="Make",
    allow_negative_inventory=True
)
config_mto.safety_stock = 0.0  # MTO 通常無安全庫存
```

### 從 ERP 數據庫動態設置

```python
def load_mrp_config_from_erp(component_id):
    """從 ERP 讀取物料主檔並創建 MRP 配置"""

    # 從數據庫讀取 (示例)
    item = db.query("""
        SELECT
            component_id,
            lead_time,
            procurement_type,
            mrp_type,
            safety_stock,
            lot_size_rule,
            fixed_lot_size
        FROM materials
        WHERE component_id = ?
    """, component_id)

    # 根據 MRP 類型決定是否允許負庫存
    allow_negative = item['mrp_type'] in ['MTO', 'PTO', 'Phantom', 'Service']

    config = MrpConfig(
        component_id=item['component_id'],
        lead_time_days=item['lead_time'],
        procurement_type=item['procurement_type'],
        allow_negative_inventory=allow_negative
    )

    config.safety_stock = item['safety_stock']
    config.lot_sizing_rule = item['lot_size_rule']

    if item['fixed_lot_size']:
        config.fixed_lot_size = item['fixed_lot_size']

    return config
```

### SAP 集成範例

```python
def sap_to_mrp_config(sap_material):
    """從 SAP 物料主檔轉換為 MRP 配置"""

    # SAP MRP Type 對應
    mrp_type_mapping = {
        'PD': False,  # MRP - 不允許負庫存
        'VB': False,  # Manual reorder point - 不允許負庫存
        'VM': False,  # Forecast-based - 不允許負庫存
        'VV': False,  # Consumption-based - 不允許負庫存
        'ND': True,   # No planning - 允許負庫存
        'X0': True,   # External - 允許負庫存
    }

    allow_negative = mrp_type_mapping.get(
        sap_material['MRP_TYPE'],
        False  # 預設保守
    )

    config = MrpConfig(
        component_id=sap_material['MATNR'],
        lead_time_days=sap_material['PLIFZ'],
        procurement_type="Make" if sap_material['BESKZ'] == 'E' else "Buy",
        allow_negative_inventory=allow_negative
    )

    # SAP 安全庫存
    config.safety_stock = float(sap_material['EISBE'])

    # SAP 批量規則
    if sap_material['DISLS'] == 'FX':  # 固定批量
        config.lot_sizing_rule = "FOQ"
        config.fixed_lot_size = float(sap_material['BSTFE'])
    elif sap_material['DISLS'] == 'EX':  # 批對批
        config.lot_sizing_rule = "LotForLot"

    return config
```

## 測試案例

### Rust 測試

```rust
#[test]
fn test_allow_negative_inventory() {
    // 不允許負庫存
    let result_strict = NettingCalculator::calculate(
        &demands,
        &supplies,
        Decimal::from(30),    // 初始庫存
        Decimal::from(10),    // 安全庫存
        &time_buckets,
        false,  // 不允許負庫存
    ).unwrap();

    // 預計庫存 = 30 - 100 = -70
    // 淨需求 = 10 - (-70) = 80
    assert_eq!(result_strict[0].net_requirement, Decimal::from(80));

    // 允許負庫存
    let result_relaxed = NettingCalculator::calculate(
        &demands,
        &supplies,
        Decimal::from(30),
        Decimal::from(10),    // 被忽略
        &time_buckets,
        true,  // 允許負庫存
    ).unwrap();

    // 預計庫存 = 30 - 100 = -70
    // 淨需求 = 70（只補足到 0）
    assert_eq!(result_relaxed[0].net_requirement, Decimal::from(70));
}
```

### Python 測試

```python
def test_negative_inventory_modes():
    """測試負庫存模式"""

    # 場景：庫存 30，需求 100
    # 不允許負庫存：應產生 80 的淨需求（補到安全庫存 10）
    config_strict = MrpConfig(
        "TEST-001", 3, "Make",
        allow_negative_inventory=False
    )
    config_strict.safety_stock = 10.0

    # 允許負庫存：應產生 70 的淨需求（只補到 0）
    config_relaxed = MrpConfig(
        "TEST-002", 3, "Make",
        allow_negative_inventory=True
    )
    config_relaxed.safety_stock = 10.0

    # 執行 MRP...
    # assert_net_requirement(config_strict, 80)
    # assert_net_requirement(config_relaxed, 70)
```

## 決策樹

```
物料類型?
│
├─ 有實體庫存? ──→ 是 ──→ allow_negative_inventory = False
│                        (MTS, 原物料, 半成品)
│
├─ 客製化/專案? ──→ 是 ──→ allow_negative_inventory = True
│                        (MTO, 工程專案)
│
├─ 虛擬件?      ──→ 是 ──→ allow_negative_inventory = True
│                        (Phantom, BOM 階層)
│
└─ 服務類?      ──→ 是 ──→ allow_negative_inventory = True
                         (工時, 服務, 無形資產)
```

## 注意事項

1. **預設值選擇**
   - 預設為 `false`（不允許負庫存）
   - 符合大多數製造業場景
   - 保守策略，避免缺料風險

2. **與安全庫存的關係**
   - `allow_negative_inventory = false`: 尊重安全庫存
   - `allow_negative_inventory = true`: 忽略安全庫存
   - MTO 模式通常設置 `safety_stock = 0`

3. **ERP 集成建議**
   - 從 ERP MRP Type 自動推導
   - 提供手動覆寫選項
   - 記錄參數來源（自動 vs 手動）

4. **性能考量**
   - 允許負庫存可減少計劃訂單數量
   - 適合高週轉、低庫存的場景
   - 不影響計算速度

## 版本歷史

- **v0.1.0**: 初始實現
  - 添加 `allow_negative_inventory` 參數
  - Rust 核心邏輯實現
  - Python FFI 綁定
  - 完整測試覆蓋

## 參考資料

- [MRP 計算邏輯](./MRP_CALCULATION.md)
- [批量規則](./LOT_SIZING.md)
- [Python API 文檔](./PYTHON_API.md)
