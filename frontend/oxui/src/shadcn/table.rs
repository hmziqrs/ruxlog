use dioxus::prelude::*;

/// Properties for the Table component
#[derive(Props, PartialEq, Clone)]
pub struct TableProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// Table container and table element
#[component]
pub fn Table(props: TableProps) -> Element {
    let mut class = vec!["w-full caption-bottom text-sm".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        div {
            "data-slot": "table-container",
            class: "relative w-full overflow-x-auto",
            table {
                "data-slot": "table",
                class: class.join(" "),
                {props.children}
            }
        }
    }
}

/// Properties for table header component
#[derive(Props, PartialEq, Clone)]
pub struct TableHeaderProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// TableHeader component
#[component]
pub fn TableHeader(props: TableHeaderProps) -> Element {
    let mut class = vec!["[&_tr]:border-b".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        thead {
            "data-slot": "table-header",
            class: class.join(" "),
            {props.children}
        }
    }
}

/// Properties for table body component
#[derive(Props, PartialEq, Clone)]
pub struct TableBodyProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// TableBody component
#[component]
pub fn TableBody(props: TableBodyProps) -> Element {
    let mut class = vec!["[&_tr:last-child]:border-0".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        tbody {
            "data-slot": "table-body",
            class: class.join(" "),
            {props.children}
        }
    }
}

/// Properties for table footer component
#[derive(Props, PartialEq, Clone)]
pub struct TableFooterProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// TableFooter component
#[component]
pub fn TableFooter(props: TableFooterProps) -> Element {
    let mut class = vec!["bg-muted/50 border-t font-medium [&>tr]:last:border-b-0".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        tfoot {
            "data-slot": "table-footer",
            class: class.join(" "),
            {props.children}
        }
    }
}

/// Properties for table row component
#[derive(Props, PartialEq, Clone)]
pub struct TableRowProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
    /// Whether the row is selected
    #[props(default)]
    pub selected: bool,
}

/// TableRow component
#[component]
pub fn TableRow(props: TableRowProps) -> Element {
    let mut class = vec!["hover:bg-muted/50 border-b transition-colors".to_string()];

    if props.selected {
        class.push("bg-muted".to_string());
    }

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        tr {
            "data-slot": "table-row",
            "data-state": if props.selected { "selected" } else { "" },
            class: class.join(" "),
            {props.children}
        }
    }
}

/// Properties for table head component
#[derive(Props, PartialEq, Clone)]
pub struct TableHeadProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// TableHead component
#[component]
pub fn TableHead(props: TableHeadProps) -> Element {
    let mut class = vec![
        "text-foreground h-10 px-2 text-left align-middle font-medium whitespace-nowrap"
            .to_string(),
        "[&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]".to_string(),
    ];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        th {
            "data-slot": "table-head",
            class: class.join(" "),
            {props.children}
        }
    }
}

/// Properties for table cell component
#[derive(Props, PartialEq, Clone)]
pub struct TableCellProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// TableCell component
#[component]
pub fn TableCell(props: TableCellProps) -> Element {
    let mut class = vec![
        "p-2 align-middle whitespace-nowrap".to_string(),
        "[&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]".to_string(),
    ];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        td {
            "data-slot": "table-cell",
            class: class.join(" "),
            {props.children}
        }
    }
}

/// Properties for table caption component
#[derive(Props, PartialEq, Clone)]
pub struct TableCaptionProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// TableCaption component
#[component]
pub fn TableCaption(props: TableCaptionProps) -> Element {
    let mut class = vec!["text-muted-foreground mt-4 text-sm".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        caption {
            "data-slot": "table-caption",
            class: class.join(" "),
            {props.children}
        }
    }
}
