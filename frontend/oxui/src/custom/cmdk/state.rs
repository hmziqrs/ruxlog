use dioxus::{logger::tracing, prelude::*};
use im::HashMap;

// So idea is we will always have consumable list from the props
// we can use that list to compute filtered groups at runtime
// and then we can use that to render the list
// also on every search reset active index to 0
// Also have a way for disabled items
// as soon as input is focused we should set the active index to 0

#[derive(Props, PartialEq, Clone)]
pub struct CommandListProps {
    pub data: Vec<CommandListItem>,
    // pub initial_selected: CommandListItem,
    // pub children: Element,
    // #[props(optional)]
    // pub on_select: fn(CommandListItem),
    #[props(optional)]
    pub reset_on_select: bool,

    pub groups: Vec<String>,
}

#[derive(PartialEq, Clone)]
pub struct CommandListGroup {
    pub label: String,
    pub id: String,
    pub items: Vec<CommandInternalListItem>,
}

impl CommandListGroup {
    pub fn new(label: String, id: String, items: Vec<CommandInternalListItem>) -> Self {
        Self { label, items, id }
    }
}

#[derive(PartialEq, Clone)]
pub struct CommandInternalListItem {
    pub label: String,
    pub value: String,
    pub group_id: String,
    pub disabled: bool,
    pub index: usize,
}

impl CommandInternalListItem {
    pub fn new(group_id: String, label: String, disabled: bool, index: usize) -> Self {
        let value = label.clone().to_lowercase().replace(" ", "-");

        Self {
            label,
            value,
            group_id,
            disabled,
            index,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct CommandListItem {
    pub label: String,
    pub group_id: String,
    pub disabled: bool,
}

impl CommandListItem {
    pub fn new(group_id: String, label: String, disabled: bool) -> Self {
        Self {
            label,
            group_id,
            disabled,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct CommandContext {
    pub search: String,
    pub active_index: usize,
    pub selected_index: usize,
    pub groups: Vec<CommandListGroup>,
    pub is_empty: bool,

    // This shouldn't be exposed
    max_index: usize,
    internal_groups: Vec<String>,
    list: Vec<CommandListItem>,
    reset_on_select: bool,
}

impl CommandContext {
    pub fn new(groups: Vec<String>, list: Vec<CommandListItem>, reset: Option<bool>) -> Self {
        let mut g_map: HashMap<String, CommandListGroup> = HashMap::new();

        for item in list.clone() {
            let group = g_map.entry(item.group_id.clone()).or_insert_with(|| {
                CommandListGroup::new(item.group_id.clone(), item.group_id.clone(), Vec::new())
            });

            group.items.push(CommandInternalListItem::new(
                item.group_id,
                item.label,
                item.disabled,
                0,
            ));
        }

        let mut max_index: usize = 0;
        let mut parsed_groups = Vec::<CommandListGroup>::new();
        let mut index = 0;

        for (_, mut v) in g_map {
            max_index += v.items.len();
            for item in &mut v.items {
                item.index = index;
                index += 1;
            }
            parsed_groups.push(v);
        }

        Self {
            search: String::new(),
            active_index: 0,
            selected_index: 0,
            groups: parsed_groups,
            is_empty: if list.len() > 0 { false } else { true },

            max_index,
            internal_groups: groups,
            list: list,
            reset_on_select: reset.unwrap_or(false),
        }
    }

    pub fn set_search(&mut self, search: String) {
        tracing::info!("YOO ?");
        self.search = search;

        self.active_index = 0;

        self.compute_internal_groups();
    }

    fn compute_internal_groups(&mut self) {
        let mut g_map: HashMap<String, CommandListGroup> = HashMap::new();

        for item in self.list.clone() {
            if self.search.is_empty()
                || item
                    .label
                    .to_lowercase()
                    .contains(&self.search.to_lowercase())
            {
                let group = g_map.entry(item.group_id.clone()).or_insert_with(|| {
                    CommandListGroup::new(item.group_id.clone(), item.group_id.clone(), Vec::new())
                });

                group.items.push(CommandInternalListItem::new(
                    item.group_id,
                    item.label,
                    item.disabled,
                    0,
                ));
            }
        }

        let mut max_index: usize = 0;
        let mut parsed_groups = Vec::<CommandListGroup>::new();
        let mut index = 0;

        for (_, mut v) in g_map {
            max_index += v.items.len();
            for item in &mut v.items {
                item.index = index;
                index += 1;
            }
            parsed_groups.push(v);
        }

        self.groups = parsed_groups;
        self.max_index = max_index;
        self.is_empty = self.groups.is_empty();
    }

    pub fn set_next_index(&mut self) {
        if self.active_index < self.max_index - 1 {
            self.active_index += 1;
        } else {
            self.active_index = 0;
        }
    }

    pub fn set_prev_index(&mut self) {
        if self.active_index > 0 {
            self.active_index -= 1;
        } else {
            self.active_index = self.max_index - 1;
        }
    }

    pub fn set_active_index(&mut self, index: usize) {
        if index == 0 {
            self.active_index = 0;
        } else if index >= self.groups.len() {
            self.active_index = self.max_index;
        } else {
            self.active_index = index;
        }
    }
}
