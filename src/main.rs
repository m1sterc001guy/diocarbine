mod components;
mod db;
mod multimint;

use std::{fmt::Display, sync::Arc};

use components::{dashboard::Dashboard, join::JoinFederationForm};
use dioxus::prelude::*;
use fedimint_core::config::FederationId;
use multimint::Multimint;
use tokio::sync::RwLock;

const MAIN_CSS: Asset = asset!("/assets/main.css");

static MULTIMINT: GlobalSignal<Arc<RwLock<Option<Multimint>>>> =
    Global::new(|| Arc::new(RwLock::new(None)));

fn main() {
    dioxus::launch(app);
}

async fn load_multimint() -> Arc<RwLock<Option<Multimint>>> {
    if MULTIMINT().read().await.is_none() {
        *MULTIMINT.write() = Arc::new(RwLock::new(Some(
            Multimint::new().await.expect("Could not create multimint"),
        )));
    }

    MULTIMINT()
}

#[component]
pub fn app() -> Element {
    let sidebar_items = use_signal(|| Vec::new());
    let mut selected_federation = use_signal(|| None::<FederationSelector>);

    let load_items = {
        to_owned![sidebar_items];
        move || {
            spawn({
                to_owned![sidebar_items];
                async move {
                    let multimint = load_multimint().await;
                    let mm = multimint.read().await;
                    if let Some(mm) = mm.as_ref() {
                        let names = mm.federations().await;
                        sidebar_items.set(names);
                    }
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

                button {
                    class: "add-button",
                    onclick: move |_| {
                        selected_federation.set(None);
                    },
                    "+"
                }

                match selected_federation() {
                    Some(selector) => rsx! {
                        Dashboard { federation_info: selector }
                    },
                    None => rsx! {
                        JoinFederationForm {
                            on_join_success: move |selector| {
                                load_items();
                                selected_federation.set(Some(selector));
                            }
                        }
                    }
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
