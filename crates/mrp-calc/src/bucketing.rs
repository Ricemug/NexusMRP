//! 時間分桶

use chrono::NaiveDate;
use mrp_core::{Demand, Supply};

/// 時間分桶策略
#[derive(Debug, Clone, Copy)]
pub enum BucketingStrategy {
    /// 每日分桶
    Daily,
    /// 每週分桶
    Weekly,
    /// 每月分桶
    Monthly,
}

/// 時間分桶計算器
pub struct BucketingCalculator;

impl BucketingCalculator {
    /// 創建時間桶（基於需求和供應的日期範圍）
    pub fn create_time_buckets(
        demands: &[Demand],
        supplies: &[Supply],
        _planning_horizon_days: u32,
    ) -> Vec<NaiveDate> {
        let mut dates = Vec::new();

        // 收集所有需求日期
        for demand in demands {
            if !dates.contains(&demand.required_date) {
                dates.push(demand.required_date);
            }
        }

        // 收集所有供應日期
        for supply in supplies {
            if !dates.contains(&supply.available_date) {
                dates.push(supply.available_date);
            }
        }

        // 排序日期並去重
        dates.sort();
        dates.dedup();

        // 只返回有需求/供應的日期，不創建每日桶
        // 註：planning_horizon_days 參數保留供未來使用
        dates
    }

    /// 創建固定週期的時間桶
    pub fn create_buckets_by_strategy(
        start_date: NaiveDate,
        end_date: NaiveDate,
        strategy: BucketingStrategy,
    ) -> Vec<NaiveDate> {
        let mut buckets = Vec::new();
        let mut current = start_date;

        while current <= end_date {
            buckets.push(current);

            current = match strategy {
                BucketingStrategy::Daily => current
                    .succ_opt()
                    .expect("日期溢出"),
                BucketingStrategy::Weekly => current
                    .checked_add_signed(chrono::Duration::weeks(1))
                    .expect("日期溢出"),
                BucketingStrategy::Monthly => {
                    // 簡化實現：加 30 天
                    current
                        .checked_add_signed(chrono::Duration::days(30))
                        .expect("日期溢出")
                }
            };
        }

        buckets
    }

    /// 合併重複的日期桶
    pub fn merge_buckets(buckets: &mut Vec<NaiveDate>) {
        buckets.sort();
        buckets.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_buckets_daily() {
        let start = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 10, 5).unwrap();

        let buckets = BucketingCalculator::create_buckets_by_strategy(
            start,
            end,
            BucketingStrategy::Daily,
        );

        assert_eq!(buckets.len(), 5);
        assert_eq!(buckets[0], start);
        assert_eq!(buckets[4], end);
    }

    #[test]
    fn test_merge_buckets() {
        let mut buckets = vec![
            NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 2).unwrap(),
        ];

        BucketingCalculator::merge_buckets(&mut buckets);

        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0], NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
        assert_eq!(buckets[1], NaiveDate::from_ymd_opt(2025, 10, 2).unwrap());
        assert_eq!(buckets[2], NaiveDate::from_ymd_opt(2025, 10, 3).unwrap());
    }
}
