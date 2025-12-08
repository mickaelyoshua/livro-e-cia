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
