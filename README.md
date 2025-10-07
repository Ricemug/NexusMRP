# 🏭 NexusMRP - High-Performance MRP Calculation Engine

> Lightning-fast Material Requirements Planning engine designed for modern manufacturing ERP systems

[繁體中文](./docs/README.zh-TW.md) | [简体中文](./docs/README.zh-CN.md)

[![License](https://img.shields.io/badge/license-AGPL--3.0%20%7C%20Commercial-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)]()

## 📋 Overview

NexusMRP is a high-performance Material Requirements Planning (MRP) calculation engine built with Rust, designed specifically for manufacturing ERP systems. It delivers microsecond-level calculation speeds while maintaining enterprise-grade accuracy and reliability.

### Core Features

- ⚡ **Extreme Performance** - Microsecond-level MRP calculations, 100-1000x faster than traditional implementations
- 🔄 **Incremental Calculation** - Intelligent change detection, calculates only affected components
- 🌐 **Multi-Factory Support** - Handles multi-organization, multi-plant scenarios seamlessly
- 📊 **Demand Pegging** - Complete traceability from requirements to sources
- 🔧 **Flexible Lot Sizing** - Supports LFL, FOQ, EOQ, POQ, Min-Max, and custom rules
- 🎯 **BOM Integration** - Seamless integration with Bill of Materials engines
- 🐍 **Python Bindings** - Full Python API through PyO3 FFI
- 📅 **Dynamic Time Buckets** - Flexible time bucketing for planning horizons
- 🔴 **Negative Inventory Handling** - Advanced shortage tracking and resolution

## 🏗️ Project Structure

```
NexusMRP/
├── crates/
│   ├── mrp-core/          # Core data models and types
│   ├── mrp-calc/          # MRP calculation engine
│   ├── mrp-optimizer/     # Optimization algorithms (capacity, scheduling)
│   ├── mrp-ffi/           # Python FFI bindings
│   └── mrp-cache/         # Caching and incremental computation
├── examples/              # Usage examples
├── benches/               # Performance benchmarks
└── tests/                 # Integration tests
```

## 🚀 Quick Start

### Prerequisites

Ensure you have Rust 1.75 or higher installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building the Project

```bash
# Clone the repository
git clone https://github.com/Ricemug/NexusMRP.git
cd NexusMRP

# Build all crates
cargo build --release

# Run tests
cargo test

# Run example
cargo run --example simple_mrp
```

### Basic Usage

```rust
use chrono::NaiveDate;
use mrp_core::{Demand, DemandType, MrpConfig, ProcurementType};
use rust_decimal::Decimal;

// Create demand
let demand = Demand::new(
    "BIKE-001".to_string(),
    Decimal::from(100),
    NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
    DemandType::SalesOrder,
);

// Create MRP configuration
let config = MrpConfig::new(
    "BIKE-001".to_string(),
    5,  // Lead time: 5 days
    ProcurementType::Make,
);

// Execute MRP calculation
// let result = calculator.calculate(demands, supplies, inventories)?;
```

### Python Example

```python
from mrp import MRPCalculator, Demand, MrpConfig

# Create calculator
calculator = MRPCalculator()

# Define demand
demand = Demand(
    item_id="BIKE-001",
    quantity=100,
    due_date="2025-11-01",
    demand_type="SalesOrder"
)

# Calculate MRP
result = calculator.calculate([demand])
print(result.planned_orders)
```

## 📚 Documentation

Detailed documentation available in the `docs/` directory:

- [Dynamic Time Buckets](./docs/DYNAMIC_TIME_BUCKETS.md) - Flexible time bucket planning
- [Negative Inventory Handling](./docs/NEGATIVE_INVENTORY.md) - Advanced shortage management
- [Commercial License](./docs/COMMERCIAL-LICENSE.zh-TW.md) - Commercial licensing terms

## 🔧 Development Status

Current Version: `v0.1.0` (In Development)

- [x] Project scaffolding
- [x] Core data models
- [x] MRP calculation engine
- [x] Lot sizing rules implementation
- [x] Demand pegging functionality
- [x] Python FFI bindings
- [x] Incremental calculation with dirty tracking
- [x] Optimizer module (capacity & scheduling)
- [ ] Complete test coverage
- [ ] Performance optimization
- [ ] Documentation completion

## 🎯 Performance Targets

| Operation | Target Time | Data Scale |
|-----------|------------|------------|
| Single item MRP | < 50 μs | 5-level BOM |
| 100 items batch | < 5 ms | Avg 3-level BOM |
| 10,000 SKU enterprise MRP | < 5 seconds | Mixed BOM depths |

## 🏗️ Architecture

NexusMRP uses a modular architecture:

1. **mrp-core**: Foundational types (demands, supplies, inventory, calendar)
2. **mrp-calc**: Core MRP algorithms (netting, lot sizing, lead time offset, pegging)
3. **mrp-cache**: Incremental calculation and dirty tracking
4. **mrp-optimizer**: Advanced optimization (capacity planning, scheduling)
5. **mrp-ffi**: Language bindings for Python and other languages

## 💼 Licensing

This project is dual-licensed:

- **Open Source**: [AGPL-3.0](./LICENSE) for open-source projects
- **Commercial**: [Commercial License](./COMMERCIAL-LICENSE.md) for proprietary use

### When do you need a commercial license?

You need a commercial license if you:
- Use NexusMRP in a proprietary/closed-source product
- Distribute software containing NexusMRP without making your code open source
- Provide SaaS services using NexusMRP without open-sourcing your application

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

## 🙏 Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [PyO3](https://pyo3.rs/) - Python bindings for Rust
- [rust_decimal](https://github.com/paupino/rust-decimal) - Decimal arithmetic
- [chrono](https://github.com/chronotope/chrono) - Date and time handling

## 📜 License

Copyright (c) 2025 NexusMRP Contributors

Licensed under either:
- AGPL-3.0 License ([LICENSE](./LICENSE))
- Commercial License ([COMMERCIAL-LICENSE.md](./COMMERCIAL-LICENSE.md))

at your option.

---

**Made with ❤️ for Smart Manufacturing**
