//! 工作日曆模型

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

/// 工作日曆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkCalendar {
    /// 工作日（週一到週日，true表示工作日）
    /// 索引 0 = 週一, 1 = 週二, ..., 6 = 週日
    pub working_days: [bool; 7],

    /// 節假日列表
    pub holidays: Vec<NaiveDate>,

    /// 日曆ID
    pub calendar_id: String,
}

impl WorkCalendar {
    /// 創建新的工作日曆（預設週一到週五為工作日）
    pub fn new(calendar_id: String) -> Self {
        Self {
            working_days: [true, true, true, true, true, false, false], // 週一到週五
            calendar_id,
            holidays: Vec::new(),
        }
    }

    /// 創建 24/7 日曆（所有日子都是工作日）
    pub fn new_24_7(calendar_id: String) -> Self {
        Self {
            working_days: [true; 7],
            calendar_id,
            holidays: Vec::new(),
        }
    }

    /// 建構器模式：設置工作日
    pub fn with_working_days(mut self, working_days: [bool; 7]) -> Self {
        self.working_days = working_days;
        self
    }

    /// 建構器模式：添加節假日
    pub fn with_holidays(mut self, holidays: Vec<NaiveDate>) -> Self {
        self.holidays = holidays;
        self
    }

    /// 添加節假日
    pub fn add_holiday(&mut self, date: NaiveDate) {
        if !self.holidays.contains(&date) {
            self.holidays.push(date);
            self.holidays.sort();
        }
    }

    /// 檢查是否為工作日
    pub fn is_working_day(&self, date: NaiveDate) -> bool {
        // 檢查是否為節假日
        if self.holidays.contains(&date) {
            return false;
        }

        // 檢查是否為工作日
        let weekday_index = date.weekday().num_days_from_monday() as usize;
        self.working_days[weekday_index]
    }

    /// 計算工作日（向前推算）
    pub fn add_working_days(&self, start_date: NaiveDate, days: u32) -> NaiveDate {
        let mut current = start_date;
        let mut remaining = days;

        while remaining > 0 {
            current = current
                .succ_opt()
                .expect("日期溢出");
            if self.is_working_day(current) {
                remaining -= 1;
            }
        }

        current
    }

    /// 計算工作日（向後推算）
    pub fn subtract_working_days(&self, start_date: NaiveDate, days: u32) -> NaiveDate {
        let mut current = start_date;
        let mut remaining = days;

        while remaining > 0 {
            current = current
                .pred_opt()
                .expect("日期溢出");
            if self.is_working_day(current) {
                remaining -= 1;
            }
        }

        current
    }

    /// 計算兩個日期之間的工作日數量
    pub fn working_days_between(&self, start: NaiveDate, end: NaiveDate) -> u32 {
        let mut count = 0;
        let mut current = start;

        while current < end {
            current = current.succ_opt().expect("日期溢出");
            if self.is_working_day(current) {
                count += 1;
            }
        }

        count
    }

    /// 獲取下一個工作日
    pub fn next_working_day(&self, date: NaiveDate) -> NaiveDate {
        self.add_working_days(date, 1)
    }

    /// 獲取上一個工作日
    pub fn previous_working_day(&self, date: NaiveDate) -> NaiveDate {
        self.subtract_working_days(date, 1)
    }
}

impl Default for WorkCalendar {
    fn default() -> Self {
        Self::new("DEFAULT".to_string())
    }
}

/// 排班表資料結構（用於從 ERP 系統載入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftSchedule {
    /// 日曆ID
    pub calendar_id: String,
    /// 工作日配置（週一=0, 週二=1, ..., 週日=6）
    pub working_days: Vec<bool>,
    /// 國定假日
    pub holidays: Vec<NaiveDate>,
}

impl WorkCalendar {
    /// 從排班表創建工作日曆
    ///
    /// # 範例：從 ERP 系統載入排班表
    ///
    /// ```
    /// use mrp_core::WorkCalendar;
    ///
    /// // 假設從數據庫或 API 取得工廠排班
    /// // 週一到週六上班，週日休息
    /// let working_days_vec = vec![true, true, true, true, true, true, false];
    /// let holidays_vec = vec![];
    ///
    /// let mut working_days = [false; 7];
    /// for (i, &is_working) in working_days_vec.iter().enumerate() {
    ///     if i < 7 {
    ///         working_days[i] = is_working;
    ///     }
    /// }
    ///
    /// let calendar = WorkCalendar::new("FACTORY-A".to_string())
    ///     .with_working_days(working_days)
    ///     .with_holidays(holidays_vec);
    /// ```
    pub fn from_shift_data(
        calendar_id: String,
        working_days_vec: Vec<bool>,
        holidays: Vec<NaiveDate>,
    ) -> Self {
        let mut working_days = [false; 7];
        for (i, &is_working) in working_days_vec.iter().enumerate() {
            if i < 7 {
                working_days[i] = is_working;
            }
        }

        Self {
            working_days,
            holidays,
            calendar_id,
        }
    }

    /// 創建降級日曆（當無法取得排班表時使用）
    ///
    /// 策略：使用 24/7 日曆確保 MRP 計算不會因為缺少排班資料而中斷
    pub fn fallback_calendar() -> Self {
        Self::new_24_7("FALLBACK-24/7".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_calendar() {
        let calendar = WorkCalendar::new("TEST".to_string());
        assert_eq!(calendar.calendar_id, "TEST");

        // 週一到週五應該是工作日
        let monday = NaiveDate::from_ymd_opt(2025, 10, 6).unwrap(); // 假設是週一
        assert!(calendar.is_working_day(monday));
    }

    #[test]
    fn test_add_working_days() {
        let calendar = WorkCalendar::new("TEST".to_string());

        // 2025-10-06 是週一
        let start = NaiveDate::from_ymd_opt(2025, 10, 6).unwrap();

        // 加 5 個工作日應該到週一（跳過週末）
        let result = calendar.add_working_days(start, 5);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 10, 13).unwrap());
    }

    #[test]
    fn test_subtract_working_days() {
        let calendar = WorkCalendar::new("TEST".to_string());

        // 2025-10-13 是週一
        let start = NaiveDate::from_ymd_opt(2025, 10, 13).unwrap();

        // 減 5 個工作日應該回到上週一
        let result = calendar.subtract_working_days(start, 5);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 10, 6).unwrap());
    }

    #[test]
    fn test_holidays() {
        let mut calendar = WorkCalendar::new("TEST".to_string());

        let holiday = NaiveDate::from_ymd_opt(2025, 10, 10).unwrap(); // 國慶日
        calendar.add_holiday(holiday);

        assert!(!calendar.is_working_day(holiday));
    }

    #[test]
    fn test_24_7_calendar() {
        let calendar = WorkCalendar::new_24_7("24/7".to_string());

        let saturday = NaiveDate::from_ymd_opt(2025, 10, 11).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2025, 10, 12).unwrap();

        assert!(calendar.is_working_day(saturday));
        assert!(calendar.is_working_day(sunday));
    }

    #[test]
    fn test_working_days_between() {
        let calendar = WorkCalendar::new("TEST".to_string());

        let start = NaiveDate::from_ymd_opt(2025, 10, 6).unwrap(); // 週一
        let end = NaiveDate::from_ymd_opt(2025, 10, 13).unwrap(); // 下週一

        // 週一到週五 = 5天，跳過週末
        let count = calendar.working_days_between(start, end);
        assert_eq!(count, 5);
    }
}
