use sea_orm::{DbConn, DbErr, EntityTrait, PaginatorTrait, Select};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<Model> {
    // Changed T to Model for clarity, and data type to Vec<Model>
    pub data: Vec<Model>,
    pub page: Page,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub page_number: u64,
    pub page_size: u64,
    pub total_items: u64,
    pub total_pages: u64,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl Page {
    pub fn new(page_number: u64, page_size: u64, total_items: u64) -> Self {
        // Handle page_size = 0 to avoid division by zero
        let total_pages = if page_size == 0 {
            0 // Or perhaps 1 if total_items > 0, depending on desired behavior
        } else {
            (total_items + page_size - 1) / page_size
        };

        // Ensure page_number is at least 1 for calculations
        let current_page = if page_number == 0 { 1 } else { page_number };

        Self {
            page_number: current_page, // Use the adjusted page number
            page_size,
            total_items,
            total_pages,
            // Ensure has_next_page is false if total_pages is 0
            has_next_page: total_pages > 0 && current_page < total_pages,
            has_previous_page: current_page > 1,
        }
    }
}

// Use async_trait to simplify async trait method definition
#[async_trait::async_trait]
pub trait Paginate<E>
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Send + Sync + Serialize + for<'de> Deserialize<'de>, // Add serde bounds
    Self: Sized + Send,
{
    async fn paginate(
        self,
        conn: &DbConn,
        page_number: u64,
        page_size: u64,
    ) -> Result<PagedResult<<E as EntityTrait>::Model>, DbErr>;
}

// Implement the trait for SeaORM's Select
#[async_trait::async_trait]
impl<E> Paginate<E> for Select<E>
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Send + Sync + Serialize + for<'de> Deserialize<'de>, // Add serde bounds here too
{
    async fn paginate(
        self,
        conn: &DbConn,
        page_number: u64,
        page_size: u64,
    ) -> Result<PagedResult<<E as EntityTrait>::Model>, DbErr> {
        // Ensure page_number is at least 1 for user-facing display and calculations
        let current_page = if page_number == 0 { 1 } else { page_number };
        // SeaORM's fetch_page is 0-indexed
        let page_index = current_page - 1;

        // Create the paginator using the PaginatorTrait method explicitly
        let paginator = PaginatorTrait::paginate(self, conn, page_size);

        // Get total items first
        let total_items = paginator.num_items().await?;

        // Fetch the items for the requested page
        // Handle potential error if page_index is out of bounds (though num_items should help)
        let items = paginator.fetch_page(page_index).await?;

        // Create the page metadata
        let page = Page::new(current_page, page_size, total_items);

        // Construct the final result
        Ok(PagedResult { data: items, page })
    }
}
