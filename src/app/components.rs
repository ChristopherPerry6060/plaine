#![allow(non_snake_case)]
use dioxus::prelude::*;
use plaine::db::Mongo;

// LoginForm props.
#[derive(Props, PartialEq, Default, Clone, Debug)]
pub struct LoginFormProp {
    user: String,
    pass: String,
}

impl LoginFormProp {
    /// Return an `Option<(String, String)>` from [`Self`].
    fn clone_get(&self) -> (String, String) {
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

#[inline_props]
pub fn LoginChecker<'a>(cx: Scope, check: &'a UseFuture<anyhow::Result<()>>) -> Element<'a> {
    match check.value()? {
        Ok(_) => cx.render(rsx! { Link{ to: "/home", "Continue!"} }),
        Err(e) => cx.render(rsx! {"{e}"}),
    }
}

#[inline_props]
pub fn Login(cx: Scope) -> Element {
    let creds = use_ref(cx, LoginFormProp::default);
    let check = use_future(cx, creds, |creds| async move {
        to_owned![creds];
        let (user, pw) = creds.with(|x| x.clone_get());
        Mongo::new()
            .set_user(&user)
            .set_password(&pw)
            .set_database("items")
            .build()
            .await?
            .check_credentials()
            .await
    });
    cx.render(rsx! {
        LoginForm { creds: creds, },
        LoginChecker { check: check, },
    })
}

/// Component to display a login form with a "User" and "Password" field.
///
/// Event handler is passed as a prop , allowing hooks to be implemented
/// by choice of the caller.
#[inline_props]
fn LoginForm<'a>(cx: Scope, creds: &'a UseRef<LoginFormProp>) -> Element<'a> {
    let (username, password) = creds.with(|i| i.clone_get());

    cx.render(rsx! {
        form {
            onsubmit: move |evt| creds.with_mut(|inner| {
                let u = evt.values.get("user").unwrap();
                let p = evt.values.get("pass").cloned().unwrap_or_default();
                inner.set(&u, &p);
            }),
            label {r#for: "fuser", "Username: ", },
            input { r#type: "text", id: "fuser", name: "user", value: "{username}", }, br{},
            label {r#for: "fpass", "Password: ", },
            input { r#type: "password", id: "fpass", name: "pass", value: "{password}", }, br{},
            input { r#type: "submit", },
        },
    })
}

