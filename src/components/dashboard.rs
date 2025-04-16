use dioxus::prelude::*;
use fedimint_core::Amount;

use crate::{load_multimint, FederationSelector};

#[component]
pub fn Dashboard(federation_info: FederationSelector) -> Element {
    let balance = use_resource(move || async move {
        let multimint = load_multimint().await;
        let mm = multimint.read().await;
        if let Some(mm) = mm.as_ref() {
            mm.balance(&federation_info.federation_id).await
        } else {
            Amount::ZERO
        }
    });

    rsx! {
        div {
            class: "dashboard",
            h3 { "Balance" }
            match balance() {
                Some(bal) => rsx! {
                    p { class: "balance-text", "{bal}" }
                },
                None => rsx! {
                    div { class: "spinner" }
                }
            }
            div {
                class: "button-row",
                button {
                    class: "send-button",
                    onclick: |_| {
                        println!("Send clicked");
                        // open send flow...
                    },
                    "Send"
                }
                button {
                    class: "receive-button",
                    onclick: |_| {
                        println!("Receive clicked");
                        // open receive flow...
                    },
                    "Receive"
                }
            }
        }
    }
}
