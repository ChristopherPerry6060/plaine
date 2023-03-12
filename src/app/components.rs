#![allow(non_snake_case)]
use dioxus::prelude::*;
use std::rc::Rc;


// LoginForm props.
//
// A username and password.
#[derive(Props, PartialEq, Default, Clone)]
pub(super) struct Creds {
    user: Option<String>,
    pass: Option<String>,
}

impl TryFrom<&Rc<FormData>> for Creds {
    type Error = anyhow::Error;

    fn try_from(value: &Rc<FormData>) -> Result<Self, Self::Error> {
        let user = value.values.get("user").cloned();
        let pass = value.values.get("pass").cloned();
        Ok(Creds { user, pass })
    }
}

impl Creds {
    pub(super) fn get(&self) -> Option<(&str, &str)> {
        let user = &self.user.as_ref()?;
        let pass = &self.pass.as_ref()?;
        Some((user, pass))
    }
}

#[inline_props]
pub(super) fn DocumentCount<'a>(cx: Scope, count: &'a UseFuture<Option<u64>>) -> Element {
    if let Some(number) = count.value()? {
        cx.render(rsx! {
            h1 { "{number}" }
        })
    } else {
        None
    }
}

/// Component to display a login form with a "User" and "Password" field.
///
/// Event handler is passed as a prop , allowing hooks to be implemented
/// by choice of the caller.
#[inline_props]
pub(super) fn LoginForm<'a>(
    cx: Scope,
    creds: &'a Creds,
    on_submit: EventHandler<'a, FormEvent>,
) -> Element<'a> {
    let (username, password) = creds.get().to_owned().unwrap_or_default();

    cx.render(rsx! {
        form {
            onsubmit: move |event| on_submit.call(event),
            label {r#for: "fuser", "Username: ", },
            input { r#type: "text", id: "fuser", name: "user", value: "{username}", }, br{},
            label {r#for: "fpass", "Password: ", },
            input { r#type: "password", id: "fpass", name: "pass", value: "{password}", }, br{},
            input { r#type: "submit", },
        },
    })
}
