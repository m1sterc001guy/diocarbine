mod db;
mod components;
mod multimint;

use std::{fmt::Display, sync::Arc};

use components::{dashboard::Dashboard, join::JoinFederationForm};
use dioxus::prelude::*;
use fedimint_core::config::FederationId;
use multimint::Multimint;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(app);
}

#[component]
pub fn app() -> Element {
    let sidebar_items = use_signal(|| Arc::new(Vec::new()));
    let mut selected_federation = use_signal(|| None::<FederationSelector>);

    let load_items = {
        to_owned![sidebar_items];
        move || {
            spawn({
                to_owned![sidebar_items];
                async move {
                    let multimint = Multimint::new().await.expect("Could not create multimint");
                    let names = multimint.federation_names().await;
                    sidebar_items.set(Arc::new(names));
                }
            });
        }
    };

    use_effect(move || {
        load_items();
    });

    rsx! {
        link { rel: "stylesheet", href: "{MAIN_CSS}" }
        div {
            class: "container",
            // Sidebar
            div {
                class: "sidebar",
                h2 { class: "sidebar-title", "Federations" }
                ul {
                    class: "sidebar-list",
                    for item in sidebar_items().iter().cloned() {
                        li {
                            class: "sidebar-item",
                            onclick: move |_| {
                                selected_federation.set(Some(item.clone()));
                            },
                            "{item}"
                        }
                    }
                }
            }

            // Main content
            div {
                class: "main",
                JoinFederationForm {
                    on_join_success: move |_| load_items()
                }
                
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct FederationSelector {
    federation_name: String,
    federation_id: FederationId,
}

impl Display for FederationSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.federation_name)
    }
}
