#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]
use dioxus::prelude::*;
use dioxus_desktop::{launch_cfg, tao::window::Theme, Config, WindowBuilder};
use dioxus_router::{Link, Route, Router};

mod components;
use components::{CredentialStatus, Creds, LoginForm};
use plaine::db::Mongo;

const STYLESHEET: &str = include_str!("assets/style.css");

fn main() {
    let head = format!("<style>{STYLESHEET}</style>");

    let win = WindowBuilder::new()
        .with_title("Plaine")
        .with_theme(Some(Theme::Dark));

    let cfg = Config::new().with_window(win).with_custom_head(head);
    launch_cfg(App, cfg);
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        Login{ },
    })
}

fn Login<'a>(cx: Scope<'a>) -> Element<'a> {
    let creds = use_ref(cx, Creds::default);
    let check = use_future(cx, creds, |x| async move {
        let (user, pw) = x.with(|x| x.clone_get());
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
        CredentialStatus { check: check, },
    })
}
