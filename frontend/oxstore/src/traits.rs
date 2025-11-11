use crate::{PaginatedList, StateFrame, ListQuery};
use dioxus::prelude::*;

pub trait ListStore<T, Q>
where
    T: Clone + PartialEq + 'static,
    Q: ListQuery,
{
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<T>>>;

    fn fetch_list(&self) -> impl std::future::Future<Output = ()>;

    fn fetch_list_with_query(&self, query: Q) -> impl std::future::Future<Output = ()>;
}
