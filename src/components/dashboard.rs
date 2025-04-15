use dioxus::prelude::*;
use fedimint_core::Amount;

use crate::{FederationSelector, Multimint};

#[component]
pub fn Dashboard(federation_info: FederationSelector) -> Element {
    let balance = use_signal(|| Amount::ZERO);
    
    use_effect(move || {
        spawn({
            to_owned![balance, federation_info];
            async move {
                let multimint = Multimint::new().await.expect("Could not create Multimint");
                let bal = multimint.balance(&federation_info.federation_id).await;
                balance.set(bal);
            }
        });
    });

    rsx! {
        div {
            class: "dashboard",
            h3 { "Balance" }
            p { "{balance()} sats" }
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