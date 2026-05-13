use serde::{Deserialize, Serialize};

/// Paginated list with navigation and iteration support
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginatedList<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

impl<T> PaginatedList<T> {
    pub fn has_next_page(&self) -> bool {
        self.page * self.per_page < self.total
    }

    pub fn has_previous_page(&self) -> bool {
        self.page > 1
    }
}

impl<T> std::ops::Deref for PaginatedList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for PaginatedList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> IntoIterator for PaginatedList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── has_next_page ──

    #[test]
    fn has_next_page_true_on_middle_page() {
        let list = PaginatedList {
            data: vec![1, 2, 3],
            total: 9,
            page: 1,
            per_page: 3,
        };
        // page 1, 3 items shown, 9 total => 1*3=3 < 9 => true
        assert!(list.has_next_page());
    }

    #[test]
    fn has_next_page_false_on_last_page() {
        let list = PaginatedList {
            data: vec![7, 8, 9],
            total: 9,
            page: 3,
            per_page: 3,
        };
        // page 3, 3*3=9 < 9 => false
        assert!(!list.has_next_page());
    }

    #[test]
    fn has_next_page_false_on_empty_list() {
        let list: PaginatedList<i32> = PaginatedList {
            data: vec![],
            total: 0,
            page: 1,
            per_page: 20,
        };
        // 1*20=20 < 0 => false
        assert!(!list.has_next_page());
    }

    #[test]
    fn has_next_page_false_when_exactly_full() {
        let list = PaginatedList {
            data: vec![1, 2],
            total: 2,
            page: 1,
            per_page: 2,
        };
        // 1*2=2 < 2 => false
        assert!(!list.has_next_page());
    }

    // ── has_previous_page ──

    #[test]
    fn has_previous_page_false_on_first_page() {
        let list = PaginatedList {
            data: vec![1, 2, 3],
            total: 9,
            page: 1,
            per_page: 3,
        };
        assert!(!list.has_previous_page());
    }

    #[test]
    fn has_previous_page_true_on_page_two() {
        let list = PaginatedList {
            data: vec![4, 5, 6],
            total: 9,
            page: 2,
            per_page: 3,
        };
        assert!(list.has_previous_page());
    }

    #[test]
    fn has_previous_page_true_on_page_greater_than_one() {
        let list: PaginatedList<i32> = PaginatedList {
            data: vec![],
            total: 100,
            page: 5,
            per_page: 20,
        };
        assert!(list.has_previous_page());
    }

    // ── Deref ──

    #[test]
    fn deref_returns_inner_data() {
        let list = PaginatedList {
            data: vec![10, 20, 30],
            total: 3,
            page: 1,
            per_page: 10,
        };
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], 10);
        assert_eq!(list[2], 30);
    }

    // ── IntoIterator ──

    #[test]
    fn into_iter_yields_all_items() {
        let list = PaginatedList {
            data: vec!["a", "b", "c"],
            total: 3,
            page: 1,
            per_page: 10,
        };
        let collected: Vec<&str> = list.into_iter().collect();
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[test]
    fn into_iter_empty_list() {
        let list: PaginatedList<i32> = PaginatedList {
            data: vec![],
            total: 0,
            page: 1,
            per_page: 10,
        };
        let collected: Vec<i32> = list.into_iter().collect();
        assert!(collected.is_empty());
    }
}