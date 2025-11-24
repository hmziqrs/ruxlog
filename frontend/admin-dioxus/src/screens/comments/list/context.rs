use dioxus::prelude::*;
use oxstore::ListQuery;
use ruxlog_shared::store::{CommentListQuery, FlagFilter, HiddenFilter};

#[derive(Clone)]
pub struct CommentListContext {
    pub selected_ids: Signal<Vec<i32>>,
    pub selected_user_id: Signal<Option<i32>>,
    pub selected_post_id: Signal<Option<i32>>,
    pub selected_flag_filter: Signal<FlagFilter>,
}

impl CommentListContext {
    pub fn new() -> Self {
        Self {
            selected_ids: use_signal(|| Vec::new()),
            selected_user_id: use_signal(|| None),
            selected_post_id: use_signal(|| None),
            selected_flag_filter: use_signal(|| FlagFilter::All),
        }
    }

    pub fn apply_filters(&mut self, filters: &mut Signal<CommentListQuery>) {
        let mut q = filters.peek().clone();
        q.set_page(1);

        q.user_id = self.selected_user_id.peek().clone();
        q.post_id = self.selected_post_id.peek().clone();
        q.flag_filter = Some(self.selected_flag_filter.peek().clone());

        filters.set(q);
    }

    pub fn clear_all_filters(&mut self, filters: &mut Signal<CommentListQuery>) {
        let mut q = filters.peek().clone();
        q.set_page(1);
        q.user_id = None;
        q.post_id = None;
        q.flag_filter = Some(FlagFilter::All);
        q.hidden_filter = None;
        q.set_search(None);
        filters.set(q);
        self.selected_user_id.set(None);
        self.selected_post_id.set(None);
        self.selected_flag_filter.set(FlagFilter::All);
    }

    pub fn clear_user_filter(&mut self, filters: &mut Signal<CommentListQuery>) {
        self.selected_user_id.set(None);
        self.apply_filters(filters);
    }

    pub fn clear_post_filter(&mut self, filters: &mut Signal<CommentListQuery>) {
        self.selected_post_id.set(None);
        self.apply_filters(filters);
    }

    pub fn set_user_filter(
        &mut self,
        filters: &mut Signal<CommentListQuery>,
        user_id: Option<i32>,
    ) {
        self.selected_user_id.set(user_id);
        self.apply_filters(filters);
    }

    pub fn set_post_filter(
        &mut self,
        filters: &mut Signal<CommentListQuery>,
        post_id: Option<i32>,
    ) {
        self.selected_post_id.set(post_id);
        self.apply_filters(filters);
    }

    pub fn set_flag_filter(
        &mut self,
        filters: &mut Signal<CommentListQuery>,
        flag_filter: FlagFilter,
    ) {
        self.selected_flag_filter.set(flag_filter);
        self.apply_filters(filters);
    }

    pub fn active_filter_count(&self, filters: &Signal<CommentListQuery>) -> usize {
        let q = filters.read();
        let mut count = 0;
        if q.user_id.is_some() {
            count += 1;
        }
        if q.post_id.is_some() {
            count += 1;
        }
        if q.flag_filter.is_some() && q.flag_filter != Some(FlagFilter::All) {
            count += 1;
        }
        if matches!(
            q.hidden_filter,
            Some(HiddenFilter::Hidden) | Some(HiddenFilter::All)
        ) {
            count += 1;
        }
        count
    }

    pub fn toggle_comment_selection(&mut self, comment_id: i32) {
        let mut ids = self.selected_ids.peek().clone();
        if ids.contains(&comment_id) {
            ids.retain(|id| *id != comment_id);
        } else {
            ids.push(comment_id);
        }
        self.selected_ids.set(ids);
    }

    pub fn select_all(&mut self, comment_ids: Vec<i32>) {
        self.selected_ids.set(comment_ids);
    }

    pub fn clear_selections(&mut self) {
        self.selected_ids.set(Vec::new());
    }

    pub fn is_all_selected(&self, total_ids: &[i32]) -> bool {
        let selected = self.selected_ids.read();
        if total_ids.is_empty() || selected.is_empty() {
            return false;
        }
        total_ids.iter().all(|id| selected.contains(id))
    }

    pub fn toggle_select_all(&mut self, comment_ids: Vec<i32>) {
        if self.is_all_selected(&comment_ids) {
            self.clear_selections();
        } else {
            self.select_all(comment_ids);
        }
    }
}

pub fn use_comment_list_context() -> CommentListContext {
    use_context::<CommentListContext>()
}
