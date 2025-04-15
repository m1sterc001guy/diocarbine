mod db;
mod components;
mod multimint;

use std::sync::Arc;

use components::join::JoinFederationForm;
use dioxus::prelude::*;
use multimint::Multimint;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(app);
}

#[component]
pub fn app() -> Element {
    let sidebar_items = use_signal(|| Arc::new(Vec::new()));

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
                    for item in sidebar_items().iter() {
                        li {
                            class: "sidebar-item",
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
