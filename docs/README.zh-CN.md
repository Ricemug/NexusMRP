# 🏭 MRP 计算引擎

> 高性能物料需求计划计算引擎，专为制造业 ERP 系统设计

[English](../README.md) | [繁體中文](./README.zh-TW.md)

[![授权](https://img.shields.io/badge/license-AGPL--3.0%20%7C%20Commercial-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![测试](https://img.shields.io/badge/tests-passing-brightgreen.svg)]()

## 🚀 特色功能

- ⚡ **超高速度**：微秒级 MRP 计算，比传统实现快 100-1000 倍
- 🔧 **SAP/Oracle 兼容**：为企业级 ERP 系统量身打造
- 🌐 **多语言支持**：提供 Python/C/C++/Java 的 FFI 绑定
- 💾 **智能缓存**：双层缓存（内存 + 持久化）达到最佳性能
- 🔄 **增量计算**：智能检测变更，只计算受影响的部分
- 📊 **完整 MRP 功能**：
  - 净需求计算（Netting）
  - 批量规则（LFL, FOQ, EOQ, POQ, Min-Max）
  - 提前期偏移（Lead Time Offsetting）
  - 需求追溯（Pegging）
  - BOM 展开集成
  - 多工厂支持

## 📦 安装方式

### Rust

```toml
[dependencies]
mrp-core = { git = "https://github.com/Ricemug/mrp" }
mrp-calc = { git = "https://github.com/Ricemug/mrp" }
```

### Python (通过 FFI)

```bash
# 即将推出
pip install mrp-engine
```

## 🎯 快速开始

### 基本示例

```rust
use chrono::NaiveDate;
use mrp_core::*;
use mrp_calc::*;
use rust_decimal::Decimal;

// 创建需求
let demand = Demand::new(
    "BIKE-001".to_string(),
    Decimal::from(100),
    NaiveDate::from_ymd_opt(2025, 11, 10).unwrap(),
    DemandType::SalesOrder,
);

// 创建 MRP 配置
let config = MrpConfig::new(
    "BIKE-001".to_string(),
    7,  // 提前期 7 天
    ProcurementType::Make,
)
.with_lot_sizing_rule(LotSizingRule::LotForLot)
.with_safety_stock(Decimal::from(10));

// 执行 MRP 计算
let calculator = MrpCalculator::new(bom_graph, configs, calendar);
let result = calculator.calculate(demands, supplies, inventories)?;

println!("计划订单数量: {}", result.planned_orders.len());
```

## 💼 授权

本项目采用双授权模式：

- **开源授权**：[AGPL-3.0](../LICENSE) 用于开源项目
- **商业授权**：[商业授权](./COMMERCIAL-LICENSE.zh-CN.md) 用于专有软件

商业授权咨询，请联系：xiaoivan1@proton.me

## 🤝 贡献

欢迎贡献！请参阅 [贡献指南](./CONTRIBUTING.zh-CN.md)。

可使用以下语言贡献：
- English
- 繁體中文
- 简体中文

## 💖 支持本项目

如果您觉得本项目有用，请考虑支持开发：

[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support-ff5e5b?logo=ko-fi)](https://ko-fi.com/ivanh0906)

## 📜 授权声明

Copyright (c) 2025 MRP Calculation Engine Contributors

可选择以下任一授权：
- AGPL-3.0 授权（[LICENSE](../LICENSE)）
- 商业授权（[COMMERCIAL-LICENSE.md](../COMMERCIAL-LICENSE.md)）

---

**为制造业打造，用 ❤️ 开发**
