use dioxus::{logger::tracing::info, prelude::*};

use crate::{FederationSelector, Multimint};

#[component]
pub fn Dashboard(federation_info: FederationSelector) -> Element {
    let balance = use_resource(move || async move {
        info!("Creating multimint...");
        let multimint = Multimint::new().await.expect("Could not create Multimint");
        info!("Getting balance...");
        multimint.balance(&federation_info.federation_id).await
        //futures_timer::Delay::new(std::time::Duration::from_secs(2)).await;
        //fedimint_core::Amount::from_msats(10000)
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
