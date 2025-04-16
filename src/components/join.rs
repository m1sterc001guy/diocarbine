use std::time::Duration;

use dioxus::prelude::*;
use fedimint_core::task::sleep;

use crate::load_multimint;

#[component]
pub fn JoinFederationForm(on_join_success: EventHandler<()>) -> Element {
    let mut input_value = use_signal(|| String::new());
    let error_message = use_signal(|| None::<String>); // Add signal for errors

    let on_join = {
        to_owned![input_value, error_message, on_join_success];
        move || {
            spawn(async move {
                let multimint = load_multimint().await;
                let mut mm = multimint.write().await;
                if let Some(mm) = mm.as_mut() {
                    match mm.join_federation(input_value()).await {
                        Ok(_) => {
                            input_value.set(String::new());
                            error_message.set(None); // clear errors
                            on_join_success.call(());
                        }
                        Err(_) => {
                            error_message.set(Some(format!("Could not join federation")));
                            spawn({
                                to_owned![error_message];
                                async move {
                                    sleep(Duration::from_secs(4)).await;
                                    error_message.set(None);
                                }
                            });
                        }
                    }
                }
            });
        }
    };

    rsx! {
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

            // Toast-like error message
            if let Some(msg) = error_message() {
                div {
                    class: "toast-error",
                    "{msg}"
                }
            }
        }
    }
}
