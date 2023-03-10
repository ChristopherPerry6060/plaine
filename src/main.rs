#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::{html::label, prelude::*};

fn main() {
    // launch the dioxus app in a webview
    dioxus_desktop::launch(FormCredentials);
}

// define a component that renders a div with the text "Hello, world!"
fn FormCredentials(cx: Scope) -> Element {
    let username = use_state(cx, || String::default());
    let password = use_state(cx, || String::default());
    cx.render(rsx! {
        form {
                onsubmit: move |event| {
                    username.set(event.values.get("user").unwrap().clone());
                    password.set(event.values.get("pw").unwrap().clone());
                },
                input { name: "user", value: "{username}", }, br{},
                input { name: "pw", value: "{password}", }, br{},
                input { r#type: "submit", },
            },
        h1 { "{username}" },
        h1 { "{password}" },
    })
}
