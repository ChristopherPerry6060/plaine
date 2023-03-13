#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]
use fermi::prelude::*;
use mongodb::{bson::Document, Cursor};
use std::time::Duration;

use dioxus::prelude::*;
use dioxus_desktop::{launch_cfg, tao::window::Theme, Config, WindowBuilder};
use dioxus_router::{Link, Route, Router};

mod components;
use components::{ItemTable, Login};
use plaine::db::Mongo;
use tokio_stream::StreamExt;

const STYLESHEET: &str = include_str!("assets/style.css");
pub static USER: Atom<String> = |_| String::new();
pub static PASS: Atom<String> = |_| String::new();

fn main() {
    let head = format!("<style>{STYLESHEET}</style>");
    let win = WindowBuilder::new()
        .with_title("Plaine")
        .with_theme(Some(Theme::Dark));
    let cfg = Config::new().with_window(win).with_custom_head(head);
    launch_cfg(App, cfg);
}

enum Action {
    Login(String),
}
enum Response {
    None,
    Authentication(anyhow::Result<()>),
    Cursor(anyhow::Result<Cursor<Document>>),
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    use_init_atom_root(cx);
    let thing = use_state(cx, || Response::None);
    use_coroutine(cx, |rx: UnboundedReceiver<Action>| {
        to_owned![thing];
        async move { todo!() }
    });

    cx.render(rsx! {
         Router {
             Route { to: "/", Login {} },
         },
    })
}
