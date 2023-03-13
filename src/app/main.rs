#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]
use dioxus::prelude::*;
use dioxus_desktop::{launch_cfg, tao::window::Theme, Config, WindowBuilder};
use dioxus_router::{Link, Route, Router};

mod components;
use plaine::db::Mongo;
use components::{ItemTable, Login};

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
         Router {
             Route { to: "/", Login {} },
         },
    })
}

    cx.render(rsx! {
        LoginForm { creds: creds, },
        CredentialStatus { check: check, },
    })
}
