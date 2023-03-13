#![allow(non_snake_case)]
use dioxus::prelude::*;
use std::rc::Rc;


// LoginForm props.
//
// A username and password.
#[derive(Props, PartialEq, Default, Clone, Debug)]
pub(super) struct Creds {
    user: String,
    pass: String,
}

impl Creds {
    /// Return an `Option<(String, String)>` from [`Self`].
    fn get(&self) -> (&str, &str) {
        (self.user.as_ref(), self.pass.as_ref())
    }

    /// Return an `Option<(String, String)>` from [`Self`].
    pub(super) fn clone_get(&self) -> (String, String) {
        let user = self.user.clone();
        let pass = self.pass.clone();
        (user, pass)
    }

    /// Mutate [`Self`] with the given `user` and `pass`.
    fn set(&mut self, user: &str, pass: &str) {
        self.user = user.to_string();
        self.pass = pass.to_string();
    }
}

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
