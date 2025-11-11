use dioxus::prelude::*;

/// UI cell types for skeleton table rows
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UICellType {
    /// Avatar cell with circular indicator and primary/secondary text
    Avatar,
    /// Default cell with simple rectangular placeholder
    Default,
    /// Badge-like cell with rounded placeholder  
    Badge,
    /// Action cell with button-like placeholder
    Action,
}

/// Configuration for a single skeleton cell
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonCellConfig {
    pub cell_type: UICellType,
    pub class: String,
    pub colspan: Option<u32>,
}

/// Props for the skeleton table rows component
#[derive(Props, Clone, PartialEq)]
pub struct SkeletonTableRowsProps {
    /// Number of skeleton rows to render
    pub row_count: usize,
    /// Configuration for each cell in a row
    pub cells: Vec<SkeletonCellConfig>,
    /// Additional CSS class for the table row
    #[props(default)]
    pub row_class: String,
}

/// Renders skeleton loading rows for tables with configurable cell types
#[component]
pub fn SkeletonTableRows(props: SkeletonTableRowsProps) -> Element {
    rsx! {
        { (0..props.row_count).map(|_| {
            let row_class = if props.row_class.is_empty() {
                "border-b border-border/60".to_string()
            } else {
                props.row_class.clone()
            };

            rsx! {
                tr { class: "{row_class}",
                    { props.cells.iter().map(|cell_config| {
                        let colspan_attr = cell_config.colspan.map(|n| format!("{}", n));

                        rsx! {
                            td {
                                class: "{cell_config.class}",
                                colspan: colspan_attr,
                                { render_skeleton_cell(cell_config.cell_type) }
                            }
                        }
                    }) }
                }
            }
        }) }
    }
}

/// Renders the skeleton content based on the cell type
fn render_skeleton_cell(cell_type: UICellType) -> Element {
    match cell_type {
        UICellType::Avatar => rsx! {
            div { class: "flex items-center gap-3",
                div { class: "h-3.5 w-3.5 shrink-0 rounded-full bg-muted animate-pulse" }
                div { class: "min-w-0 space-y-2",
                    div { class: "h-4 w-24 rounded bg-muted animate-pulse" }
                    div { class: "mt-1 h-3 w-20 rounded bg-muted animate-pulse md:hidden" }
                }
            }
        },
        UICellType::Default => rsx! {
            div { class: "h-4 w-24 rounded bg-muted animate-pulse" }
        },
        UICellType::Badge => rsx! {
            div { class: "h-6 w-20 rounded-full bg-muted animate-pulse" }
        },
        UICellType::Action => rsx! {
            div { class: "flex items-center justify-end",
                div { class: "h-8 w-8 rounded bg-muted animate-pulse" }
            }
        },
    }
}

/// Helper function to create common skeleton configurations
impl SkeletonCellConfig {
    /// Creates an avatar cell config with standard styling
    pub fn avatar() -> Self {
        Self {
            cell_type: UICellType::Avatar,
            class: "py-3 px-4".to_string(),
            colspan: None,
        }
    }

    /// Creates a default cell config with standard styling, optionally hidden on mobile
    pub fn default(hidden_mobile: bool) -> Self {
        let class = if hidden_mobile {
            "hidden py-3 px-4 md:table-cell".to_string()
        } else {
            "py-3 px-4".to_string()
        };

        Self {
            cell_type: UICellType::Default,
            class,
            colspan: None,
        }
    }

    /// Creates a badge cell config with standard styling
    pub fn badge() -> Self {
        Self {
            cell_type: UICellType::Badge,
            class: "py-3 px-4".to_string(),
            colspan: None,
        }
    }

    /// Creates an action cell config with standard styling
    pub fn action() -> Self {
        Self {
            cell_type: UICellType::Action,
            class: "py-3 px-4".to_string(),
            colspan: None,
        }
    }

    /// Creates a custom cell config
    pub fn custom(cell_type: UICellType, class: impl Into<String>) -> Self {
        Self {
            cell_type,
            class: class.into(),
            colspan: None,
        }
    }

    /// Adds colspan to the cell config
    pub fn with_colspan(mut self, colspan: u32) -> Self {
        self.colspan = Some(colspan);
        self
    }
}
