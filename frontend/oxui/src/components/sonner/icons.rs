//! Sonner Icons — minimal inline SVG set + loader bars
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdCheck, LdInfo, LdLoaderCircle, LdTriangleAlert, LdX},
    Icon,
};

fn class_attr(base: &str, extra: Option<&str>) -> String {
    if let Some(e) = extra {
        format!("{} {}", base, e)
    } else {
        base.to_string()
    }
}

pub fn icon_success(extra_class: Option<&str>) -> Element {
    let class = class_attr("sonner-icon h-5 w-5 text-foreground", extra_class);
    rsx! { Icon { icon: LdCheck, class: class } }
}

pub fn icon_info(extra_class: Option<&str>) -> Element {
    let class = class_attr("sonner-icon h-5 w-5 text-foreground", extra_class);
    rsx! { Icon { icon: LdInfo, class: class } }
}

pub fn icon_warning(extra_class: Option<&str>) -> Element {
    let class = class_attr("sonner-icon h-5 w-5 text-foreground", extra_class);
    rsx! { Icon { icon: LdTriangleAlert, class: class } }
}

pub fn icon_error(extra_class: Option<&str>) -> Element {
    let class = class_attr("sonner-icon h-5 w-5 text-foreground", extra_class);
    rsx! { Icon { icon: LdX, class: class } }
}

pub fn icon_close(extra_class: Option<&str>) -> Element {
    let class = class_attr("sonner-icon h-4 w-4", extra_class);
    rsx! { Icon { icon: LdX, class: class } }
}

/// A simple spinner using Tailwind's animate-spin class.
pub fn loader_spinner(extra_class: Option<&str>) -> Element {
    let class = class_attr("h-5 w-5 animate-spin text-foreground", extra_class);
    rsx! { Icon { icon: LdLoaderCircle, class: class } }
}
