#![allow(non_snake_case)]
use super::{PASS, USER};
use dioxus::{html::th, prelude::*};
use dioxus_router::Link;
use fermi::prelude::*;
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
pub fn Login(cx: Scope) -> Element {
    let creds = use_ref(cx, LoginFormProp::default);
    let db = use_coroutine_handle::<super::Action>(cx)?;

    let read_user = use_read(cx, USER);
    let read_pass = use_read(cx, PASS);

    cx.render(rsx! {
        LoginForm { },
        h1 { "{read_user}" },
    })
}

/// Component to display a login form with a "User" and "Password" field.
///
/// Event handler is passed as a prop , allowing hooks to be implemented
/// by choice of the caller.
#[inline_props]
fn LoginForm(cx: Scope) -> Element {
    let set_user = use_set(cx, USER);
    let set_pass = use_set(cx, PASS);
    cx.render(rsx! {
        form {
            onsubmit: move |evt| {
                evt.values.get("user").map(|x| set_user(x.clone()));
                evt.values.get("pass").map(|x| set_pass(x.clone()));
            },
            label {r#for: "fuser", "Username: ", },
            input { r#type: "text", id: "fuser", name: "user", }, br{},
            label {r#for: "fpass", "Password: ", },
            input { r#type: "password", id: "fpass", name: "pass", }, br{},
            input { r#type: "submit", },
        },
    })
}

#[inline_props]
pub(super) fn ItemTable<T>(cx: Scope, items: Vec<T>) -> Element
where
    T: plaine::Table,
{
    let hdr = items.last()?.headers().into_iter();
    let rws = items.iter().map(|x| x.row());
    cx.render(rsx! {
        table {
            tr {
                for headers in hdr {
                    td { headers }

                }
            },
            for row in rws {
                tr {
                    for data in row {
                        td { data }
                    }
                }
            }
        },
    })
}
