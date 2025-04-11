mod db;
mod multimint;

use dioxus::prelude::*;
use multimint::Multimint;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(app);
}

#[component]
fn app() -> Element {

    let fed_names = use_resource(|| async {
        let mut multimint = Multimint::new().await.expect("Could not create Multimint");
        match multimint.join_federation("fed11qgqpw9thwvaz7te3xgmjuvpwxqhrzw338q6ngve0qqqjqzg948su5g0072p7t2wmf44apvhecuqwpvgcucx4wh6p42mupfwd0a8v8z".to_string()).await {
            Ok(()) => println!("Successfully joined federation"),
            Err(e) => println!("Could not join federation: {e}"),
        };
        let mut names = Vec::new();
        for (_, client) in multimint.clients {
            names.push(client.config().await.global.federation_name().expect("No federation name").to_owned());
        }
        names
    });

    rsx! {
        link { rel: "stylesheet", href: "{MAIN_CSS}" }

        div { class: "main-layout",
            nav { class: "sidebar",
                ul {
                    match fed_names.read().clone() {
                        Some(names) => {
                            if names.is_empty() {
                                rsx! {
                                    li {
                                        button {
                                            onclick: |_| {
                                                // Federation join logic here
                                                println!("Join Federation clicked");
                                            },
                                            "Join Federation"
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    for name in names {
                                        li { "{name}" }
                                    }
                                }
                            }
                        },
                        None => rsx!(li { "Loading..." })
                    }
                }
            }

            main { class: "content-area",
                match fed_names.read().clone() {
                    Some(names) => {
                        if names.is_empty() {
                            rsx!(h1 { "No federations joined." })
                        } else {
                            // You can compute total balance here if you want
                            rsx! {
                                div { class: "balance-display",
                                    h1 { "Balance: 123_456 sats" } // Replace with real balance
                                    div { class: "actions",
                                        button { "Receive" }
                                        button { "Send" }
                                    }
                                }
                            }
                        }
                    },
                    None => rsx!(h1 { "Initializing Multimint..." })
                }
            }
        }
    }
}
