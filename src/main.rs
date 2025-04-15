mod db;
mod multimint;

use std::sync::Arc;

use dioxus::prelude::*;
use multimint::Multimint;
use rand::{seq::SliceRandom, thread_rng};

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(app);
}

async fn fetch_sidebar_items() -> Vec<String> {
    let mut rng = thread_rng();
    let all_names = vec![
        "Nebula Union",
        "Starlight Pact",
        "Quantum Ring",
        "Nova Collective",
        "Galactic Syndicate",
        "Void Alliance",
        "Cosmic Web",
        "Photon Order",
        "Eclipse Circle",
        "Wormhole Network",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();

    let count = 3 + rand::random::<usize>() % 4; // Randomly choose 3 to 6 items
    all_names
        .choose_multiple(&mut rng, count)
        .cloned()
        .collect()
}

#[component]
pub fn app() -> Element {
    let sidebar_items = use_signal(|| Arc::new(Vec::new()));
    let mut input_value = use_signal(|| String::new());

    let load_items = {
        to_owned![sidebar_items];
        move || {
            spawn({
                to_owned![sidebar_items];
                async move {
                    //let items = fetch_sidebar_items().await;
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

    let on_join = {
        to_owned![input_value];
        move || {
            spawn({
                to_owned![input_value];
                async move {
                    let multimint = Multimint::new().await.expect("Could not create multimint");
                    multimint
                        .join_federation(input_value())
                        .await
                        .expect("Could not join federation");
                    load_items();
                    input_value.set(String::new());
                }
            });
        }
    };

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
                div {
                    class: "form-area",
                    input {
                        class: "input-box",
                        r#type: "text",
                        placeholder: "Enter federation join code...",
                        value: "{input_value}",
                        oninput: move |evt| input_value.set(evt.value().clone())
                    }
                    button {
                        class: "join-button",
                        onclick: move |_| on_join(),
                        "Join Federation"
                    }
                }
            }
        }
    }
}
