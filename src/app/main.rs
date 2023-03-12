use dioxus::prelude::*;
use dioxus_desktop::{launch_cfg, tao::window::Theme, Config, WindowBuilder};
use plaine::db::{Mongo, Ready};

mod components;
use components::{Creds, DocumentCount, LoginForm};

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
