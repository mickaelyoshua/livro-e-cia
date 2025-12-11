pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub offset: i64,
}

impl Pagination {
    const DEFAULT_PAGE: i64 = 1;
    const DEFAULT_PER_PAGE: i64 = 20;
    const MAX_PER_PAGE: i64 = 100;

    pub fn new(page: Option<i64>, per_page: Option<i64>) -> Self {
        let page = page.unwrap_or(Self::DEFAULT_PAGE).max(1);
        let per_page = per_page
            .unwrap_or(Self::DEFAULT_PER_PAGE)
            .clamp(1, Self::MAX_PER_PAGE);
        let offset = (page - 1) * per_page;

        Self {
            page,
            per_page,
            offset,
        }
    }

    pub fn total_pages(&self, total_count: i64) -> i64 {
        (total_count + self.per_page - 1) / self.per_page
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_default_values_when_none_provided() {
        let p = Pagination::new(None, None);
        assert_eq!(p.page, 1);
        assert_eq!(p.per_page, 20);
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn new_accepts_custom_page_and_per_page() {
        let p = Pagination::new(Some(3), Some(50));
        assert_eq!(p.page, 3);
        assert_eq!(p.per_page, 50);
        assert_eq!(p.offset, 100); // (3-1) * 50
    }

    #[test]
    fn new_clamps_page_to_minimum_1() {
        let p = Pagination::new(Some(0), None);
        assert_eq!(p.page, 1);

        let p2 = Pagination::new(Some(-5), None);
        assert_eq!(p2.page, 1);
    }

    #[test]
    fn new_clamps_per_page_to_minimum_1() {
        let p = Pagination::new(None, Some(0));
        assert_eq!(p.per_page, 1);

        let p2 = Pagination::new(None, Some(-10));
        assert_eq!(p2.per_page, 1);
    }

    #[test]
    fn new_clamps_per_page_to_maximum_100() {
        let p = Pagination::new(None, Some(500));
        assert_eq!(p.per_page, 100);
    }

    #[test]
    fn offset_calculated_correctly() {
        // Page 1: offset 0
        assert_eq!(Pagination::new(Some(1), Some(10)).offset, 0);
        // Page 2: offset 10
        assert_eq!(Pagination::new(Some(2), Some(10)).offset, 10);
        // Page 5: offset 40
        assert_eq!(Pagination::new(Some(5), Some(10)).offset, 40);
    }

    #[test]
    fn total_pages_rounds_up() {
        let p = Pagination::new(None, Some(10));
        // 25 items / 10 per page = 3 pages (ceil)
        assert_eq!(p.total_pages(25), 3);
        // 30 items / 10 per page = 3 pages (exact)
        assert_eq!(p.total_pages(30), 3);
        // 31 items / 10 per page = 4 pages
        assert_eq!(p.total_pages(31), 4);
    }

    #[test]
    fn total_pages_handles_zero_count() {
        let p = Pagination::new(None, Some(10));
        assert_eq!(p.total_pages(0), 0);
    }

    #[test]
    fn total_pages_handles_single_item() {
        let p = Pagination::new(None, Some(10));
        assert_eq!(p.total_pages(1), 1);
    }

    #[test]
    fn total_pages_with_per_page_1() {
        let p = Pagination::new(None, Some(1));
        assert_eq!(p.total_pages(5), 5);
    }
}
