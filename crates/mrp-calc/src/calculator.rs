//! MRP 主計算器

use bom_graph::BomGraph;
use mrp_core::{Demand, Inventory, MrpConfig, Supply, WorkCalendar};
use std::collections::HashMap;

use crate::{ComponentMrpResult, MrpResult};

/// MRP 計算器
pub struct MrpCalculator {
    /// BOM 圖（來自 BOM 引擎）
    bom_graph: BomGraph,

    /// MRP 配置
    configs: HashMap<String, MrpConfig>,

    /// 工作日曆
    calendar: WorkCalendar,
}

impl MrpCalculator {
    /// 創建新的 MRP 計算器
    pub fn new(
        bom_graph: BomGraph,
        configs: HashMap<String, MrpConfig>,
        calendar: WorkCalendar,
    ) -> Self {
        Self {
            bom_graph,
            configs,
            calendar,
        }
    }

    /// 主 MRP 計算入口
    pub fn calculate(
        &self,
        demands: Vec<Demand>,
        supplies: Vec<Supply>,
        inventories: Vec<Inventory>,
    ) -> mrp_core::Result<MrpResult> {
        tracing::info!(
            "開始 MRP 計算：需求 {} 筆，供應 {} 筆，庫存 {} 筆",
            demands.len(),
            supplies.len(),
            inventories.len()
        );

        let start_time = std::time::Instant::now();

        // Step 1: 按時間分桶（Time Bucketing）
        tracing::debug!("Step 1: 時間分桶");
        let planning_horizon = self.get_max_planning_horizon();
        let time_buckets = crate::bucketing::BucketingCalculator::create_time_buckets(
            &demands,
            &supplies,
            planning_horizon,
        );
        tracing::debug!("時間桶數量: {}", time_buckets.len());

        // Step 2: 按物料分組需求/供應/庫存
        tracing::debug!("Step 2: 物料分組");
        let grouped_demands = self.group_demands_by_component(&demands);
        let grouped_supplies = self.group_supplies_by_component(&supplies);
        let inventory_map = self.create_inventory_map(&inventories);
        tracing::debug!("物料數量: {}", grouped_demands.len());

        // Step 3: 拓撲排序（依 BOM 層級，從下到上計算）
        tracing::debug!("Step 3: 拓撲排序");
        let sorted_components = self.topological_sort(&grouped_demands)?;
        tracing::debug!("排序後物料: {:?}", sorted_components);

        // Step 4: 逐物料計算 MRP（按拓撲順序）
        tracing::debug!("Step 4: 逐物料計算 MRP");
        let mut all_planned_orders = Vec::new();
        let mut dependent_demands: HashMap<String, Vec<Demand>> = HashMap::new();
        let mut processed_components: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        // 先處理有獨立需求的物料
        let mut components_to_process = sorted_components.clone();

        // 迭代處理，直到沒有新的相依需求
        while !components_to_process.is_empty() {
            let component_id = components_to_process.remove(0);

            // 避免重複處理
            if processed_components.contains(&component_id) {
                continue;
            }

            tracing::debug!("計算物料 MRP: {}", component_id);

            // 合併獨立需求和相依需求
            let mut component_demands = grouped_demands
                .get(&component_id)
                .cloned()
                .unwrap_or_default();

            if let Some(dep_demands) = dependent_demands.get(&component_id) {
                component_demands.extend(dep_demands.clone());
            }

            // 如果沒有任何需求，跳過
            if component_demands.is_empty() {
                processed_components.insert(component_id);
                continue;
            }

            // 計算該物料的 MRP
            let component_result = self.calculate_component_mrp(
                &component_id,
                &component_demands,
                &grouped_supplies,
                &inventory_map,
                &time_buckets,
            )?;

            // 收集計劃訂單
            all_planned_orders.extend(component_result.planned_orders.clone());

            // BOM 展開：為子件生成相依需求
            let child_demands = self.explode_bom(&component_id, &component_result.planned_orders)?;
            for (child_id, child_demand_list) in child_demands {
                // 將新的子件加入待處理列表
                if !processed_components.contains(&child_id)
                    && !components_to_process.contains(&child_id)
                {
                    components_to_process.push(child_id.clone());
                }

                dependent_demands
                    .entry(child_id.clone())
                    .or_insert_with(Vec::new)
                    .extend(child_demand_list);
            }

            processed_components.insert(component_id);
        }

        // Step 5: 需求追溯（Pegging）
        tracing::debug!("Step 5: 需求追溯");
        let pegging = crate::pegging::PeggingCalculator::perform(
            &all_planned_orders,
            &demands,
            crate::pegging::PeggingType::MultiLevel,
        )?;

        let mut result = MrpResult::empty();
        result.planned_orders = all_planned_orders;
        result.pegging = pegging;
        result.calculation_time_ms = Some(start_time.elapsed().as_millis());

        tracing::info!("MRP 計算完成，耗時 {:?}", start_time.elapsed());
        tracing::info!("計劃訂單數量: {}", result.planned_orders.len());

        Ok(result)
    }

    /// 單物料 MRP 計算
    fn calculate_component_mrp(
        &self,
        component_id: &str,
        component_demands: &[Demand],
        grouped_supplies: &HashMap<String, Vec<Supply>>,
        inventory_map: &HashMap<String, Inventory>,
        time_buckets: &[chrono::NaiveDate],
    ) -> mrp_core::Result<ComponentMrpResult> {
        let config = self
            .configs
            .get(component_id)
            .ok_or_else(|| mrp_core::MrpError::ConfigNotFound(component_id.to_string()))?;

        // 如果該物料不啟用 MRP，跳過
        if !config.needs_mrp() {
            tracing::debug!("物料 {} 不啟用 MRP，跳過", component_id);
            return Ok(ComponentMrpResult {
                component_id: component_id.to_string(),
                planned_orders: Vec::new(),
            });
        }

        // 獲取該物料的供應和庫存
        let component_supplies = grouped_supplies
            .get(component_id)
            .cloned()
            .unwrap_or_default();

        let initial_inventory = inventory_map
            .get(component_id)
            .map(|inv| inv.available_qty)
            .unwrap_or_else(|| rust_decimal::Decimal::ZERO);

        // 動態創建時間桶：合併基礎時間桶和該物料的實際需求/供應日期
        let component_time_buckets = self.create_component_time_buckets(
            time_buckets,
            component_demands,
            &component_supplies,
        );

        tracing::debug!(
            "物料 {} 時間桶: 基礎 {} 個, 擴展後 {} 個",
            component_id,
            time_buckets.len(),
            component_time_buckets.len()
        );

        // 計算淨需求
        let net_requirements = crate::netting::NettingCalculator::calculate(
            component_demands,
            &component_supplies,
            initial_inventory,
            config.safety_stock,
            &component_time_buckets, // 使用動態時間桶
            config.allow_negative_inventory, // 從配置中讀取是否允許負庫存
        )?;

        // 應用批量規則，生成計劃訂單
        let planned_orders = crate::lot_sizing::LotSizingCalculator::apply(
            component_id,
            &net_requirements,
            config,
            &self.calendar,
        )?;

        tracing::debug!(
            "物料 {} 計劃訂單: {} 筆",
            component_id,
            planned_orders.len()
        );

        Ok(ComponentMrpResult {
            component_id: component_id.to_string(),
            planned_orders,
        })
    }

    /// 按物料分組需求
    fn group_demands_by_component(&self, demands: &[Demand]) -> HashMap<String, Vec<Demand>> {
        let mut grouped = HashMap::new();
        for demand in demands {
            grouped
                .entry(demand.component_id.clone())
                .or_insert_with(Vec::new)
                .push(demand.clone());
        }
        grouped
    }

    /// 按物料分組供應
    fn group_supplies_by_component(&self, supplies: &[Supply]) -> HashMap<String, Vec<Supply>> {
        let mut grouped = HashMap::new();
        for supply in supplies {
            grouped
                .entry(supply.component_id.clone())
                .or_insert_with(Vec::new)
                .push(supply.clone());
        }
        grouped
    }

    /// 創建庫存映射
    fn create_inventory_map(&self, inventories: &[Inventory]) -> HashMap<String, Inventory> {
        inventories
            .iter()
            .map(|inv| (inv.component_id.clone(), inv.clone()))
            .collect()
    }

    /// 獲取最大計劃時界（天數）
    fn get_max_planning_horizon(&self) -> u32 {
        self.configs
            .values()
            .map(|c| c.planning_horizon_days)
            .max()
            .unwrap_or(90) // 預設 90 天
    }

    /// 拓撲排序（依 BOM 層級）
    /// 返回排序後的物料列表（從子件到父件）
    fn topological_sort(
        &self,
        grouped_demands: &HashMap<String, Vec<Demand>>,
    ) -> mrp_core::Result<Vec<String>> {
        // 收集所有需要計算的物料
        let components: Vec<String> = grouped_demands.keys().cloned().collect();

        // 使用 BOM 圖進行拓撲排序
        // 簡化實現：如果沒有 BOM 關係，直接返回
        // 實際應該使用 BOM 圖的 topological_sort 方法

        // TODO: 整合 BOM 引擎的拓撲排序
        // let sorted = self.bom_graph.topological_sort(&components)?;

        // 臨時實現：直接返回原列表
        Ok(components)
    }

    /// BOM 展開：根據計劃訂單生成子件的相依需求
    fn explode_bom(
        &self,
        parent_id: &str,
        planned_orders: &[mrp_core::PlannedOrder],
    ) -> mrp_core::Result<HashMap<String, Vec<Demand>>> {
        use mrp_core::DemandType;

        let mut child_demands: HashMap<String, Vec<Demand>> = HashMap::new();

        // 如果沒有計劃訂單，直接返回
        if planned_orders.is_empty() {
            return Ok(child_demands);
        }

        // 在 BOM 圖中查找父件節點
        let parent_component_id = bom_core::ComponentId::new(parent_id);
        let parent_node = match self.bom_graph.arena().find_node(&parent_component_id) {
            Some(node) => node,
            None => {
                // 如果找不到節點，說明該物料沒有 BOM 或不在圖中
                tracing::debug!("物料 {} 不在 BOM 圖中，無子件", parent_id);
                return Ok(child_demands);
            }
        };

        // 獲取所有子件
        let children: Vec<_> = self
            .bom_graph
            .arena()
            .children(parent_node)
            .collect();

        if children.is_empty() {
            tracing::debug!("物料 {} 沒有子件", parent_id);
            return Ok(child_demands);
        }

        // 對每張計劃訂單，展開子件需求
        for order in planned_orders {
            for (child_node_idx, edge) in &children {
                let child_node = self
                    .bom_graph
                    .arena()
                    .node(*child_node_idx)
                    .ok_or_else(|| {
                        mrp_core::MrpError::BomExplosionError(
                            "無法獲取子件節點".to_string(),
                        )
                    })?;

                let child_component_id = &child_node.component_id;
                let child_id = child_component_id.as_str();

                // 計算子件需求數量 = 父件訂單數量 × 子件用量
                let child_quantity = order.quantity * edge.bom_item.quantity;

                // 計算子件需求日期（考慮父件的生產開始日期）
                // 子件需求日期 = 父件訂單日期（生產開始日）
                let child_required_date = order.order_date;

                // 創建相依需求
                let dependent_demand = Demand::new(
                    child_id.to_string(),
                    child_quantity,
                    child_required_date,
                    DemandType::Dependent,
                )
                .with_source_ref(format!("{}:{}", parent_id, order.id))
                .with_priority(order.pegging.first().map(|_p| 5).unwrap_or(5));

                child_demands
                    .entry(child_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(dependent_demand);

                tracing::debug!(
                    "BOM 展開: {} → {} (數量: {}, 日期: {})",
                    parent_id,
                    child_id,
                    child_quantity,
                    child_required_date
                );
            }
        }

        Ok(child_demands)
    }

    /// 為單一物料創建動態時間桶
    ///
    /// 合併基礎時間桶和該物料的實際需求/供應日期，確保所有相依需求日期都被包含
    fn create_component_time_buckets(
        &self,
        base_time_buckets: &[chrono::NaiveDate],
        component_demands: &[Demand],
        component_supplies: &[Supply],
    ) -> Vec<chrono::NaiveDate> {
        use std::collections::HashSet;

        // 使用 HashSet 收集所有唯一日期
        let mut dates: HashSet<chrono::NaiveDate> = base_time_buckets.iter().copied().collect();

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

    /// 獲取工作日曆引用
    pub fn calendar(&self) -> &WorkCalendar {
        &self.calendar
    }

    /// 獲取 BOM 圖引用
    pub fn bom_graph(&self) -> &BomGraph {
        &self.bom_graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mrp_core::{Demand, DemandType, Supply, SupplyType};

    #[test]
    fn test_create_calculator() {
        // TODO: 實現測試
    }

    #[test]
    fn test_dynamic_time_buckets() {
        // 測試動態時間桶：確保相依需求日期被包含

        // 創建測試用的 MrpCalculator
        let bom_graph = BomGraph::new();
        let configs = HashMap::new();
        let calendar = WorkCalendar::new("TEST".to_string());
        let calculator = MrpCalculator::new(bom_graph, configs, calendar);

        // 基礎時間桶（來自獨立需求）
        let base_buckets = vec![
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 11, 5).unwrap(),
        ];

        // 物料的實際需求（包含相依需求在新日期）
        let component_demands = vec![
            Demand::new(
                "PART-001".to_string(),
                rust_decimal::Decimal::from(100),
                NaiveDate::from_ymd_opt(2025, 11, 3).unwrap(), // 新日期
                DemandType::Dependent,
            ),
            Demand::new(
                "PART-001".to_string(),
                rust_decimal::Decimal::from(50),
                NaiveDate::from_ymd_opt(2025, 11, 8).unwrap(), // 新日期
                DemandType::Dependent,
            ),
        ];

        // 物料的供應
        let component_supplies = vec![Supply::new(
            "PART-001".to_string(),
            rust_decimal::Decimal::from(30),
            NaiveDate::from_ymd_opt(2025, 11, 7).unwrap(), // 新日期
            SupplyType::PurchaseOrder,
        )];

        // 創建動態時間桶
        let dynamic_buckets = calculator.create_component_time_buckets(
            &base_buckets,
            &component_demands,
            &component_supplies,
        );

        // 驗證：時間桶應包含所有日期
        assert_eq!(dynamic_buckets.len(), 5); // 2 基礎 + 2 需求 + 1 供應

        // 驗證排序正確
        assert_eq!(
            dynamic_buckets[0],
            NaiveDate::from_ymd_opt(2025, 11, 1).unwrap()
        );
        assert_eq!(
            dynamic_buckets[1],
            NaiveDate::from_ymd_opt(2025, 11, 3).unwrap()
        );
        assert_eq!(
            dynamic_buckets[2],
            NaiveDate::from_ymd_opt(2025, 11, 5).unwrap()
        );
        assert_eq!(
            dynamic_buckets[3],
            NaiveDate::from_ymd_opt(2025, 11, 7).unwrap()
        );
        assert_eq!(
            dynamic_buckets[4],
            NaiveDate::from_ymd_opt(2025, 11, 8).unwrap()
        );

        // 驗證沒有重複日期
        let unique_count = dynamic_buckets.len();
        let mut sorted = dynamic_buckets.clone();
        sorted.dedup();
        assert_eq!(sorted.len(), unique_count);
    }
}
