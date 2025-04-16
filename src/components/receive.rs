use dioxus::{logger::tracing::info, prelude::*};
use fedimint_core::Amount;

use crate::{load_multimint, FederationSelector};

#[component]
pub fn Receive(federation_info: FederationSelector) -> Element {
    let mut amount_msats = use_signal(|| "".to_string());
    let mut invoice = use_signal(|| None::<String>);

    let generate_invoice = move |_| {
        spawn({
            async move {
                let amount_msats = amount_msats().trim().parse::<u64>();
                match amount_msats {
                    Ok(msats) if msats > 0 => {
                        let amount = Amount::from_msats(msats);
                        let multimint = load_multimint().await;
                        let mm = multimint.read().await;
                        if let Some(mm) = mm.as_ref() {
                            match mm.receive(&federation_info.federation_id, amount).await {
                                Ok((generated_invoice, _operation_id)) => {
                                    invoice.set(Some(generated_invoice));
                                }
                                Err(e) => {
                                    info!("Receive returning error: {e}");
                                }
                            }
                        }
                    }
                    _ => {
                        invoice.set(Some("Invalid amount".to_string()));
                    }
                }
            }
        });
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
