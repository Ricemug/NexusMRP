# 🏭 MRP 計算引擎

> 高效能物料需求計劃計算引擎，專為製造業 ERP 系統設計

[English](../README.md) | [简体中文](./README.zh-CN.md)

[![授權](https://img.shields.io/badge/license-AGPL--3.0%20%7C%20Commercial-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![測試](https://img.shields.io/badge/tests-passing-brightgreen.svg)]()

## 🚀 特色功能

- ⚡ **超高速度**：微秒級 MRP 計算，比傳統實現快 100-1000 倍
- 🔧 **SAP/Oracle 相容**：為企業級 ERP 系統量身打造
- 🌐 **多語言支援**：提供 Python/C/C++/Java 的 FFI 綁定
- 💾 **智慧快取**：雙層快取（記憶體 + 持久化）達到最佳效能
- 🔄 **增量計算**：智能檢測變更，只計算受影響的部分
- 📊 **完整 MRP 功能**：
  - 淨需求計算（Netting）
  - 批量規則（LFL, FOQ, EOQ, POQ, Min-Max）
  - 提前期偏移（Lead Time Offsetting）
  - 需求追溯（Pegging）
  - BOM 展開整合
  - 多工廠支援

## 📦 安裝方式

### Rust

```toml
[dependencies]
mrp-core = { git = "https://github.com/Ricemug/mrp" }
mrp-calc = { git = "https://github.com/Ricemug/mrp" }
```

### Python (透過 FFI)

```bash
# 即將推出
pip install mrp-engine
```

### C/C++

```bash
git clone https://github.com/Ricemug/mrp
cd mrp/crates/mrp-ffi
cargo build --release
# Header: target/release/include/mrp.h
# Library: target/release/libmrp_ffi.so
```

## 🎯 快速開始

### 基本範例

```rust
use chrono::NaiveDate;
use mrp_core::*;
use mrp_calc::*;
use rust_decimal::Decimal;

// 創建需求
let demand = Demand::new(
    "BIKE-001".to_string(),
    Decimal::from(100),
    NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(),
    DemandType::SalesOrder,
);

// 創建 MRP 配置
let config = MrpConfig::new(
    "BIKE-001".to_string(),
    7,  // 提前期 7 天
    ProcurementType::Make,
)
.with_lot_sizing_rule(LotSizingRule::LotForLot)
.with_safety_stock(Decimal::from(10));

// 執行 MRP 計算
let calculator = MrpCalculator::new(bom_graph, configs, calendar);
let result = calculator.calculate(demands, supplies, inventories)?;

println!("計劃訂單數量: {}", result.planned_orders.len());
```

### Python 範例

```python
from mrp_engine import MrpConfig, MrpCalculator

# 創建配置
config = MrpConfig(
    component_id="BIKE-001",
    lead_time_days=7,
    procurement_type="Make"
)
config.safety_stock = 10.0
config.lot_sizing_rule = "LotForLot"

# 執行 MRP
# calculator = MrpCalculator(...)
# result = calculator.calculate(...)
```

## 📊 性能基準

在 AMD Ryzen 9 7950X 上測試（單執行緒）：

| 操作 | 時間 | 吞吐量 |
|------|------|--------|
| 淨需求計算 | ~15 μs | 66K ops/sec |
| 批量規則應用 | ~12 μs | 83K ops/sec |
| 完整 MRP 計算 | ~50 μs | 20K ops/sec |
| 多階 BOM 展開 | ~200 μs | 5K ops/sec |

*詳細指標請參考 [BENCHMARK_RESULTS.md](./BENCHMARK_RESULTS.md)*

## 🏗️ 架構

### Crate 結構

```
mrp/
├── mrp-core/          # 核心資料模型（SAP/Oracle 相容）
├── mrp-calc/          # 計算引擎（淨需求、批量、追溯）
├── mrp-optimizer/     # 優化演算法（產能、排程）
├── mrp-cache/         # 快取層（moka + redb）
├── mrp-ffi/           # Python FFI 綁定
└── examples/          # 使用範例
```

### 核心設計

- **動態時間桶**：自動合併相依需求日期
- **負庫存控制**：可配置的庫存策略（MTS/MTO）
- **平行計算**：拓撲排序 + 分層平行處理
- **Dirty Flag 追蹤**：大型 BOM 的增量計算

## 🔧 使用場景

- **ERP 系統**：SAP、Oracle、Dynamics 整合
- **MRP 計算**：多階 BOM 的物料需求計劃
- **供應鏈**：需求預測與補貨計劃
- **生產排程**：產能規劃與負荷平衡
- **變更管理**：工程變更影響分析

## 📖 文件

- [技術設計](../MRP_ENGINE_DESIGN.md)
- [負庫存控制](../docs/NEGATIVE_INVENTORY.md)
- [動態時間桶](../docs/DYNAMIC_TIME_BUCKETS.md)
- [批量規則](../docs/LOT_SIZING.md)
- [Python API](../docs/PYTHON_API.md)

### API 文件

本地產生 API 文件：

```bash
# 產生並在瀏覽器中開啟
cargo doc --no-deps --open

# 產生所有 crate 的文件
cargo doc --workspace --no-deps
```

文件將位於 `target/doc/mrp_core/index.html`

## 💼 授權

本專案採用雙授權模式：

- **開源授權**：[AGPL-3.0](../LICENSE) 用於開源專案
- **商業授權**：[商業授權](./COMMERCIAL-LICENSE.zh-TW.md) 用於專有軟體

商業授權諮詢，請聯繫：xiaoivan1@proton.me

## 🤝 貢獻

歡迎貢獻！請參閱 [貢獻指南](./CONTRIBUTING.zh-TW.md)。

可使用以下語言貢獻：
- English
- 繁體中文
- 简体中文

## 💖 支持本專案

如果您覺得本專案有用，請考慮支持開發：

[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support-ff5e5b?logo=ko-fi)](https://ko-fi.com/ivanh0906)

## 📜 授權聲明

Copyright (c) 2025 MRP Calculation Engine Contributors

可選擇以下任一授權：
- AGPL-3.0 授權（[LICENSE](../LICENSE)）
- 商業授權（[COMMERCIAL-LICENSE.md](../COMMERCIAL-LICENSE.md)）

## 🌟 致謝

使用以下技術構建：
- Rust
- Rayon（平行處理）
- Arena allocators
- Moka & Redb（快取）

---

**為製造業打造，用 ❤️ 開發**
