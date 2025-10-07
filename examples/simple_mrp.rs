//! 簡單 MRP 計算示例

use chrono::NaiveDate;
use mrp_core::{Demand, DemandType, MrpConfig, ProcurementType, WorkCalendar};
use rust_decimal::Decimal;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 簡單 MRP 計算示例 ===\n");

    // 創建工作日曆
    let calendar = WorkCalendar::new("DEFAULT".to_string());

    // 創建 MRP 配置
    let mut configs = HashMap::new();
    configs.insert(
        "BIKE-001".to_string(),
        MrpConfig::new("BIKE-001".to_string(), 5, ProcurementType::Make)
            .with_safety_stock(Decimal::from(10)),
    );

    // 創建需求
    let demands = vec![Demand::new(
        "BIKE-001".to_string(),
        Decimal::from(100),
        NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
        DemandType::SalesOrder,
    )
    .with_source_ref("SO-001".to_string())
    .with_priority(5)];

    println!("需求清單:");
    for demand in &demands {
        println!(
            "  - 物料: {}, 數量: {}, 需求日期: {}",
            demand.component_id, demand.quantity, demand.required_date
        );
    }

    // TODO: 執行 MRP 計算（需要 BOM 圖）
    println!("\n註：完整的 MRP 計算需要 BOM 圖數據");

    Ok(())
}
