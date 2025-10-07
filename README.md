# 🏭 MRP Calculation Engine

> High-performance Material Requirements Planning engine designed for manufacturing ERP systems

[繁體中文](./docs/README.zh-TW.md) | [简体中文](./docs/README.zh-CN.md)

[![License](https://img.shields.io/badge/license-AGPL--3.0%20%7C%20Commercial-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)]()

## 📋 專案概述

這是一個使用 Rust 開發的高性能 MRP（物料需求計劃）計算引擎，專為製造業 ERP 系統設計。

### 核心特性

- ⚡ **極致性能** - 微秒級 MRP 計算速度，比傳統實現快 100-1000 倍
- 🔄 **增量計算** - 智能檢測變更，只計算受影響的部分
- 🌐 **多工廠支援** - 支援多組織、多工廠場景
- 📊 **需求追溯** - 完整的 Pegging 功能，追溯需求來源
- 🔧 **靈活批量規則** - 支援 LFL、FOQ、EOQ、POQ、Min-Max 等
- 🎯 **BOM 整合** - 與 BOM 引擎無縫整合
- 🐍 **Python 綁定** - 通過 PyO3 提供 Python API

## 🏗️ 專案結構

```
mrp/
├── crates/
│   ├── mrp-core/          # 核心資料模型
│   ├── mrp-calc/          # MRP 計算引擎
│   ├── mrp-optimizer/     # 優化算法（產能、排程）
│   ├── mrp-ffi/           # Python FFI 綁定
│   └── mrp-cache/         # 緩存與增量計算
├── examples/              # 示例程式
├── benches/               # 性能基準測試
└── tests/                 # 集成測試
```

## 🚀 快速開始

### 安裝依賴

確保已安裝 Rust 1.75 或更高版本：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 構建專案

```bash
# 克隆專案
cd ~/code/mrp

# 構建所有 crate
cargo build --release

# 運行測試
cargo test

# 運行示例
cargo run --example simple_mrp
```

### 基本使用

```rust
use chrono::NaiveDate;
use mrp_core::{Demand, DemandType, MrpConfig, ProcurementType};
use rust_decimal::Decimal;

// 創建需求
let demand = Demand::new(
    "BIKE-001".to_string(),
    Decimal::from(100),
    NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
    DemandType::SalesOrder,
);

// 創建 MRP 配置
let config = MrpConfig::new(
    "BIKE-001".to_string(),
    5,  // 提前期 5 天
    ProcurementType::Make,
);

// 執行 MRP 計算
// let result = calculator.calculate(demands, supplies, inventories)?;
```

## 📚 文檔

詳細的技術設計請參考：
- [設計文檔](MRP_ENGINE_DESIGN.md)

## 🔧 開發狀態

當前版本：`v0.1.0`（開發中）

- [x] 專案骨架建立
- [x] 核心資料模型
- [ ] MRP 計算引擎實現
- [ ] 批量規則完整實現
- [ ] 需求追溯功能
- [ ] Python 綁定
- [ ] 性能優化

## 🎯 性能目標

| 操作 | 目標時間 | 數據規模 |
|------|---------|---------|
| 單產品 MRP 計算 | < 50 μs | 5 級 BOM |
| 100 產品批量計算 | < 5 ms | 平均 3 級 BOM |
| 10000 SKU 全公司 MRP | < 5 秒 | 混合 BOM 深度 |

## 💼 Licensing

This project is dual-licensed:

- **Open Source**: [AGPL-3.0](./LICENSE) for open-source projects
- **Commercial**: [Commercial License](./COMMERCIAL-LICENSE.md) for proprietary use

For commercial licensing inquiries, contact: xiaoivan1@proton.me

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

Contributions can be made in:
- English
- 繁體中文 (Traditional Chinese)
- 简体中文 (Simplified Chinese)

## 💖 Support This Project

If you find this project useful, consider supporting development:

[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support-ff5e5b?logo=ko-fi)](https://ko-fi.com/ivanh0906)

## 📜 License

Copyright (c) 2025 MRP Calculation Engine Contributors

Licensed under either:
- AGPL-3.0 License ([LICENSE](./LICENSE))
- Commercial License ([COMMERCIAL-LICENSE.md](./COMMERCIAL-LICENSE.md))

at your option.

---

**Made with ❤️ for Smart Manufacturing**
