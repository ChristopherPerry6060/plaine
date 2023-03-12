// Dioxus convention is CamelCase component names.
// Still keep with Rust convention for everything else.
#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_desktop::{launch_cfg, tao::window::Theme, Config, WindowBuilder};
use plaine::db::Mongo;
use std::rc::Rc;

// Representation of username and password.
type Login = Option<(String, String)>;
const STYLESHEET: &str = include_str!("./style.css");

fn main() {
    let head: String = format!("<style>{STYLESHEET}</style>");
    let win = WindowBuilder::new()
        .with_title("Plaine")
        .with_theme(Some(Theme::Dark));
    let cfg = Config::new().with_window(win).with_custom_head(head);
    launch_cfg(App, cfg);
}

fn GetCssSheet(cx: Scope) -> Element {
    cx.render(rsx! { style {} })
}

fn App(cx: Scope) -> Element {
    // login is Option<(username, password)>.
    let login: &UseRef<Login> = use_ref(cx, || None);
    let fuser = use_state(cx, || "".to_string());
    let fpass = use_state(cx, || "".to_string());
    let x = login.with(|inner| inner.clone());
    let db = use_future(cx, login, |login| async move { get_mongo_items(x).await });

    let reset_login_form = move || {
        fuser.set("".to_string());
        fpass.set("".to_string());
    };

    cx.render(rsx! {
        GetCssSheet {},
        LoginForm {
            user: fuser,
            pass: fpass,
            on_submit: move |event: Event<FormData>| {
               login.with_mut(|inner| *inner = build_login(&event.data));
               reset_login_form();
            },
        },
        MongoTable { }
    })
}

/// Component to display a login form with a "User" and "Password" field.
///
/// Event handler is passed as a prop , allowing hooks to be implemented
/// by choice of the caller.
#[inline_props]
fn LoginForm<'a>(
    cx: Scope,
    user: &'a str,
    pass: &'a str,
    on_submit: EventHandler<'a, FormEvent>,
) -> Element<'a> {
    cx.render(rsx! {
        form {
            onsubmit: move |event| on_submit.call(event),
            label {r#for: "fuser", "Username: ", },
            input { r#type: "text", id: "fuser", name: "user", value: "{user}", }, br{},
            label {r#for: "fpass", "Password: ", },
            input { r#type: "text", id: "fpass", name: "pass", value: "{pass}", }, br{},
            input { r#type: "submit", },
        },
    })
}

fn MongoTable(cx: Scope) -> Element {
    let test_vec = vec![0, 1, 3, 4, 5, 6];
    cx.render(rsx! {
        table {
            tbody {
                test_vec.into_iter().map(|x| {
                    let item = x.to_string();
                    rsx! { tr {
                        td { "this is data" },
                        td { "this is other data" },
                        td { "{item}" },
                    }}
                })
        }}
    })
}

async fn get_mongo_items(login: Login) -> Option<()> {
    let (user, pw) = login?;
    let db = Mongo::new()
        .set_user(&user)
        .set_password(&pw)
        .set_database(&"")
        .build()
        .await
        .unwrap();
    let doc = db
        .client()
        .database("items")
        .collection::<mongodb::bson::Document>("test")
        .count_documents(None, None)
        .await
        .unwrap();
    println!("herher");
    None
}

/// Extract form data and return a tuple of username and passowrd.
///
/// This exists solely as a helper function for pulling data out of an Rc.
fn build_login(data: &Rc<FormData>) -> Option<(String, String)> {
    // REFACTOR: this could be fixed up to take an arbitrary amount of data,
    // Formdata is just a HashMap so iterating on it would work easily.
    let user = data.values.get("user")?.to_owned();
    let pass = data.values.get("pass")?.to_owned();
    Some((user, pass))
}
