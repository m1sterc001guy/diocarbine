use dioxus::prelude::*;

use crate::FederationSelector;

#[component]
pub fn Receive(federation_info: FederationSelector) -> Element {
    let mut amount_msats = use_signal(|| "".to_string());
    let mut invoice = use_signal(|| None::<String>);

    let generate_invoice = move |_| {
        let amount = amount_msats().trim().parse::<u64>();
        match amount {
            Ok(msats) if msats > 0 => {
                let fake_invoice = format!("lnbc{}msat1p...", msats);
                invoice.set(Some(fake_invoice));
            }
            _ => {
                invoice.set(Some("Invalid amount".to_string()));
            }
        }
    };

    rsx! {
        div {
            class: "invoice-container",
            h2 {
                class: "invoice-title",
                "Create Lightning Invoice"
            }

            input {
                class: "invoice-input",
                r#type: "number",
                placeholder: "Amount in msats",
                value: "{amount_msats}",
                oninput: move |e| amount_msats.set(e.value().clone())
             }

             button {
                class: "invoice-button",
                onclick: generate_invoice,
                "Generate Invoice"
             }

             if let Some(invoice) = invoice() {
                div {
                    class: "invoice-output",
                    "{invoice}"
                }
             }
         }
    }
}
