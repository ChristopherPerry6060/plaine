// Dioxus convention is CamelCase component names.
// Still keep with Rust convention for everything else.
#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_desktop::{launch_cfg, tao::window::Theme, Config, WindowBuilder};
use plaine::db::{Mongo, Ready};
use std::rc::Rc;

const STYLESHEET: &str = include_str!("../assets/style.css");

// Representation of username and password.
#[derive(Props, PartialEq, Default, Clone)]
struct Creds {
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
    fn get(&self) -> Option<(&str, &str)> {
        let user = &self.user.as_ref()?;
        let pass = &self.pass.as_ref()?;
        Some((user, pass))
    }
}

fn main() {
    let head = format!("<style>{STYLESHEET}</style>");

    let win = WindowBuilder::new()
        .with_title("Plaine")
        .with_theme(Some(Theme::Dark));

    let cfg = Config::new().with_window(win).with_custom_head(head);
    launch_cfg(App, cfg);
}

fn App(cx: Scope) -> Element {
    // login is Option<(username, password)>.
    let creds = use_state(cx, Creds::default);

    // I can probably just pass the &UseRef<Creds> instead of cloning
    let count = use_future(cx, creds, |login| async move {
        db(&login)
            .await
            .unwrap_or_default()
            .count_docs_in_collection("monsoon")
            .await
            .ok()
    });

    cx.render(rsx! {
        LoginForm {
            creds: creds,
            on_submit: move |event: Event<FormData>| {
               let data = Creds::try_from(&event.data).unwrap_or_default();
               creds.set(Creds::default());
            },
        },
        DocumentCount { count: count },
    })
}

#[inline_props]
fn DocumentCount<'a>(cx: Scope, count: &'a UseFuture<Option<u64>>) -> Element {
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
fn LoginForm<'a>(
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
            input { r#type: "text", id: "fpass", name: "pass", value: "{password}", }, br{},
            input { r#type: "submit", },
        },
    })
}

async fn db(creds: &Creds) -> anyhow::Result<Mongo<Ready>> {
    let (user, pw) = creds
        .get()
        .ok_or_else(|| anyhow::anyhow!("No Credentials"))?;
    Mongo::new()
        .set_user(user)
        .set_password(pw)
        .set_database("items")
        .build()
        .await
}
