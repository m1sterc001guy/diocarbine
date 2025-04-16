use dioxus::prelude::*;

use crate::load_multimint;

#[component]
pub fn JoinFederationForm(on_join_success: EventHandler<()>) -> Element {
    let mut input_value = use_signal(|| String::new());

    let on_join = {
        to_owned![input_value, on_join_success];
        move || {
            spawn(async move {
                let multimint = load_multimint().await;
                let mut mm = multimint.write().await;
                if let Some(mm) = mm.as_mut() {
                    mm.join_federation(input_value())
                        .await
                        .expect("Could not join federation");
                    input_value.set(String::new());
                    on_join_success.call(());
                }
                //let mut multimint = load_multimint().await;
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
        }
    }
}
