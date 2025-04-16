use dioxus::prelude::*;
use fedimint_core::Amount;

use crate::{components::receive::Receive, load_multimint, FederationSelector};

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

    let mut show_receive = use_signal(|| false);

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
                    onclick: move |_| {
                        println!("Receive clicked");
                        show_receive.set(true);
                    },
                    "Receive"
                }
            }

            if show_receive() {
                div {
                    class: "modal-overlay",
                    div {
                        class: "modal-content",
                        button {
                            class: "modal-close-button",
                            onclick: move|_| show_receive.set(false),
                            "x"
                         }
                         Receive { federation_info }
                     }
                 }
            }
        }
    }
}
