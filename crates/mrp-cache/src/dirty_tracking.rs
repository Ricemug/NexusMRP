//! 髒標記追蹤

use std::collections::HashSet;

/// 髒標記追蹤器
pub struct DirtyTracker {
    dirty_components: HashSet<String>,
}

impl DirtyTracker {
    /// 創建新的追蹤器
    pub fn new() -> Self {
        Self {
            dirty_components: HashSet::new(),
        }
    }

    /// 標記物料為髒
    pub fn mark_dirty(&mut self, component_id: String) {
        self.dirty_components.insert(component_id);
    }

    /// 檢查物料是否為髒
    pub fn is_dirty(&self, component_id: &str) -> bool {
        self.dirty_components.contains(component_id)
    }

    /// 清除所有髒標記
    pub fn clear(&mut self) {
        self.dirty_components.clear();
    }

    /// 獲取所有髒物料
    pub fn get_dirty_components(&self) -> Vec<String> {
        self.dirty_components.iter().cloned().collect()
    }
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}
