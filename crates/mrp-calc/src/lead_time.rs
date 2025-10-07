//! 交期計算

use chrono::NaiveDate;
use mrp_core::WorkCalendar;

/// 交期計算器
pub struct LeadTimeCalculator;

impl LeadTimeCalculator {
    /// 計算下單日期（向後推算提前期）
    pub fn calculate_order_date(
        required_date: NaiveDate,
        lead_time_days: u32,
        calendar: &WorkCalendar,
    ) -> NaiveDate {
        calendar.subtract_working_days(required_date, lead_time_days)
    }

    /// 計算到貨日期（向前推算提前期）
    pub fn calculate_delivery_date(
        order_date: NaiveDate,
        lead_time_days: u32,
        calendar: &WorkCalendar,
    ) -> NaiveDate {
        calendar.add_working_days(order_date, lead_time_days)
    }

    /// 計算兩個日期之間的工作日數
    pub fn working_days_between(
        start: NaiveDate,
        end: NaiveDate,
        calendar: &WorkCalendar,
    ) -> u32 {
        calendar.working_days_between(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_time_calculation_24_7() {
        // 使用 24/7 日曆（工廠可能週末也生產）
        let calendar = WorkCalendar::new_24_7("FACTORY-24/7".to_string());
        let required_date = NaiveDate::from_ymd_opt(2025, 11, 1).unwrap();
        let lead_time = 5;

        let order_date = LeadTimeCalculator::calculate_order_date(
            required_date,
            lead_time,
            &calendar,
        );

        // 驗證計算出的到貨日期應該等於原始需求日期
        let delivery_date = LeadTimeCalculator::calculate_delivery_date(
            order_date,
            lead_time,
            &calendar,
        );

        assert_eq!(delivery_date, required_date);

        // 24/7 日曆：5天提前期 = 往前推 5 天
        let expected_order_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();
        assert_eq!(order_date, expected_order_date);
    }

    #[test]
    fn test_lead_time_calculation_weekday_only() {
        // 使用標準工作日曆（週一到週五）
        let calendar = WorkCalendar::default();
        // 使用週一作為需求日期，避免週末問題
        let required_date = NaiveDate::from_ymd_opt(2025, 11, 3).unwrap(); // 星期一
        let lead_time = 5;

        let order_date = LeadTimeCalculator::calculate_order_date(
            required_date,
            lead_time,
            &calendar,
        );

        // 驗證往返計算
        let delivery_date = LeadTimeCalculator::calculate_delivery_date(
            order_date,
            lead_time,
            &calendar,
        );

        assert_eq!(delivery_date, required_date);
    }

    #[test]
    fn test_lead_time_with_custom_schedule() {
        // 模擬工廠排班：週一到週六上班（週日休息）
        let mut working_days = [true; 7];
        working_days[6] = false; // 週日休息

        let calendar = WorkCalendar::new("FACTORY-6DAY".to_string())
            .with_working_days(working_days);

        let monday = NaiveDate::from_ymd_opt(2025, 11, 3).unwrap();
        let lead_time = 6;

        let order_date = LeadTimeCalculator::calculate_order_date(monday, lead_time, &calendar);

        // 往前推 6 個工作日（跳過週日）
        let delivery_date = LeadTimeCalculator::calculate_delivery_date(
            order_date,
            lead_time,
            &calendar,
        );

        assert_eq!(delivery_date, monday);
    }
}
