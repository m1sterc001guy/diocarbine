use dioxus::{logger::tracing::info, prelude::*};
use fedimint_lnv2_client::FinalSendOperationState;

use crate::{load_multimint, FederationSelector};

#[component]
pub fn Send(federation_info: FederationSelector) -> Element {
    let mut invoice = use_signal(|| String::new());
    let sending = use_signal(|| false);
    let result = use_signal(|| None::<String>);

    let on_send = {
        to_owned![invoice, sending, result];

        move |_| {
            let invoice_value = invoice().trim().to_string();
            if invoice_value.is_empty() {
                result.set(Some("Invoice cannot be empty".to_string()));
            }

            sending.set(true);
            result.set(None);

            spawn({
                to_owned![sending, result];
                async move {
                    let multimint = load_multimint().await;
                    let mm = multimint.read().await;
                    if let Some(mm) = mm.as_ref() {
                        match mm.send(&federation_info.federation_id, invoice_value).await {
                            Ok(operation_id) => {
                                result.set(Some(format!("Payment sent...")));

                                match mm
                                    .await_send(&federation_info.federation_id, operation_id)
                                    .await
                                {
                                    Ok(FinalSendOperationState::Success) => {
                                        result.set(Some(format!("Invoice paid successfully")));
                                    }
                                    Ok(_) => {
                                        result.set(Some(format!("Error when paying invoice")));
                                    }
                                    _ => {
                                        result.set(Some(format!("Unspecified error")));
                                    }
                                }

                                sending.set(false);
                            }
                            Err(e) => {
                                info!("Send returning error: {e}");
                            }
                        }
                    }
                }
            });
        }
    };

    rsx! {
        div {
            class: "invoice-container",
            h2 { class: "invoice-title", "Send Lightning Payment" }
            textarea {
                class: "invoice-input",
                rows: 4,
                value: "{invoice}",
                oninput: move |e| invoice.set(e.value().clone()),
                placeholder: "Paste Lightning Invoice..."
            }
            button {
                class: "invoice-button",
                onclick: on_send,
                disabled: "{sending()}",
                "Send"
            }

            if let Some(res) = result() {
                div {
                    class: "invoice-output",
                    "{res}"
                }
            }
        }
    }
}
