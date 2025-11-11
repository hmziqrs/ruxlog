use crate::store::PostListQuery;
use dioxus::prelude::*;
use oxstore::ListQuery;

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    Table,
    Grid,
}

#[derive(Clone)]
pub struct PostListContext {
    pub selected_ids: Signal<Vec<i32>>,
    pub view_mode: Signal<ViewMode>,
    pub selected_category_ids: Signal<Vec<i32>>,
    pub selected_tag_ids: Signal<Vec<i32>>,
    pub selected_author_ids: Signal<Vec<i32>>,
}

impl PostListContext {
    pub fn new() -> Self {
        Self {
            selected_ids: use_signal(|| Vec::new()),
            view_mode: use_signal(|| ViewMode::Table),
            selected_category_ids: use_signal(|| Vec::new()),
            selected_tag_ids: use_signal(|| Vec::new()),
            selected_author_ids: use_signal(|| Vec::new()),
        }
    }

    pub fn apply_filters(&mut self, filters: &mut Signal<PostListQuery>) {
        let mut q = filters.peek().clone();
        q.set_page(1);

        let cat_ids = self.selected_category_ids.peek().clone();
        q.category_id = if cat_ids.len() == 1 {
            Some(cat_ids[0])
        } else {
            None
        };

        let tag_ids = self.selected_tag_ids.peek().clone();
        q.tag_ids = if !tag_ids.is_empty() {
            Some(tag_ids)
        } else {
            None
        };

        let author_ids = self.selected_author_ids.peek().clone();
        q.author_id = if author_ids.len() == 1 {
            Some(author_ids[0])
        } else {
            None
        };

        filters.set(q);
    }

    pub fn clear_all_filters(&mut self, filters: &mut Signal<PostListQuery>) {
        let mut q = filters.peek().clone();
        q.set_page(1);
        q.category_id = None;
        q.tag_ids = None;
        q.author_id = None;
        q.status = None;
        q.set_search(None);
        filters.set(q);
        self.selected_category_ids.set(Vec::new());
        self.selected_tag_ids.set(Vec::new());
        self.selected_author_ids.set(Vec::new());
    }

    pub fn clear_status_filter(&mut self, filters: &mut Signal<PostListQuery>) {
        let mut q = filters.peek().clone();
        q.status = None;
        filters.set(q);
    }

    pub fn clear_category_filter(&mut self, filters: &mut Signal<PostListQuery>) {
        self.selected_category_ids.set(Vec::new());
        self.apply_filters(filters);
    }

    pub fn clear_tag_filter(&mut self, filters: &mut Signal<PostListQuery>, tag_id: i32) {
        let mut ids = self.selected_tag_ids.peek().clone();
        ids.retain(|id| *id != tag_id);
        self.selected_tag_ids.set(ids);
        self.apply_filters(filters);
    }

    pub fn clear_author_filter(&mut self, filters: &mut Signal<PostListQuery>) {
        self.selected_author_ids.set(Vec::new());
        self.apply_filters(filters);
    }

    pub fn active_filter_count(&self, filters: &Signal<PostListQuery>) -> usize {
        let q = filters.read();
        let mut count = 0;
        if q.category_id.is_some() {
            count += 1;
        }
        if let Some(tag_ids) = &q.tag_ids {
            count += tag_ids.len();
        }
        if q.author_id.is_some() {
            count += 1;
        }
        if q.status.is_some() {
            count += 1;
        }
        count
    }

    pub fn toggle_category(&mut self, filters: &mut Signal<PostListQuery>, cat_id: i32) {
        let mut ids = self.selected_category_ids.peek().clone();
        if ids.contains(&cat_id) {
            ids.retain(|id| *id != cat_id);
        } else {
            ids.clear();
            ids.push(cat_id);
        }
        self.selected_category_ids.set(ids);
        self.apply_filters(filters);
    }

    pub fn toggle_tag(&mut self, filters: &mut Signal<PostListQuery>, tag_id: i32) {
        let mut ids = self.selected_tag_ids.peek().clone();
        if ids.contains(&tag_id) {
            ids.retain(|id| *id != tag_id);
        } else {
            ids.push(tag_id);
        }
        self.selected_tag_ids.set(ids);
        self.apply_filters(filters);
    }

    pub fn toggle_author(&mut self, filters: &mut Signal<PostListQuery>, author_id: i32) {
        let mut ids = self.selected_author_ids.peek().clone();
        if ids.contains(&author_id) {
            ids.retain(|id| *id != author_id);
        } else {
            ids.clear();
            ids.push(author_id);
        }
        self.selected_author_ids.set(ids);
        self.apply_filters(filters);
    }

    pub fn toggle_post_selection(&mut self, post_id: i32) {
        let mut ids = self.selected_ids.peek().clone();
        if ids.contains(&post_id) {
            ids.retain(|id| *id != post_id);
        } else {
            ids.push(post_id);
        }
        self.selected_ids.set(ids);
    }

    pub fn clear_selections(&mut self) {
        self.selected_ids.set(Vec::new());
    }
}

pub fn use_post_list_context() -> PostListContext {
    use_context::<PostListContext>()
}
