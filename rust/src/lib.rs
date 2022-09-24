#![allow(dead_code)]
#![allow(clippy::unnecessary_unwrap)]

pub mod file_handle;
pub mod utils;

use std::rc::Rc;
use std::{cell::RefCell, fmt::Write};

use anyhow::{anyhow, Result};
use axum::http::header;
use axum::{extract::State, response::IntoResponse};
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Pool, Postgres, Row};

use utils::tech_emp;
#[allow(non_camel_case_types)]
type sstring = smallstr::SmallString<[u8; 23]>;

#[derive(Serialize)]
struct User {
    uid: i32,
    first_name: sstring,
    last_name: sstring,
}
impl User {
    #[inline(always)]
    fn fill_json_string(&self, s: &mut String) {
        _ = write!(
            s,
            "{{\"uid\":{},\"first_name\":\"{}\",\"last_name\":\"{}\"}},",
            self.uid, self.first_name, self.last_name
        );
    }
}

#[inline(always)]
async fn get_sql(pool: Pool<Postgres>) -> Result<Vec<PgRow>> {
    let db_res = sqlx::query("SELECT * from users").fetch_all(&pool).await?;
    Ok(db_res)
}

type UsersList = Rc<RefCell<Vec<User>>>;
trait UsersListInit {
    fn with_capacity(sz: usize) -> Self;
}
impl UsersListInit for UsersList {
    #[inline(always)]
    fn with_capacity(sz: usize) -> Self {
        Rc::new(RefCell::new(Vec::with_capacity(sz)))
    }
}

fn get_users(sql_rows: Vec<PgRow>) -> Rc<RefCell<Vec<User>>> {
    thread_local! {
        static USERS: UsersList = UsersList::with_capacity(1000);
    }

    USERS.with(|u| {
        let users = &mut *u.borrow_mut();
        users.clear();

        sql_rows.iter().for_each(|row| {
            let uid: i32 = row.get(0);
            let fname: &str = row.get(1);
            let lname: &str = row.get(2);
            users.push(User {
                uid,
                first_name: sstring::from(fname),
                last_name: sstring::from(lname),
            })
        });
        users.shrink_to(1000);

        u.clone()
    })
}

async fn get_resp_json(pool: Pool<Postgres>) -> Result<String> {
    let sql = get_sql(pool).await?;
    let mut resp = Vec::with_capacity(65_535);
    let users = get_users(sql);

    let writer = tech_emp::Writer(&mut resp);
    serde_json::to_writer(writer, &users).expect("no serial");

    if let Ok(resp) = String::from_utf8(resp) {
        Ok(resp)
    } else {
        Err(anyhow!("invalid utf8"))
    }
}

async fn get_resp_json_manual(pool: Pool<Postgres>) -> Result<String> {
    let sql = get_sql(pool).await?;
    let mut resp = String::with_capacity(65_535);

    resp.push('[');

    get_users(sql).borrow().iter().for_each(|user| {
        user.fill_json_string(&mut resp);
    });

    resp.pop();
    resp.push(']');

    Ok(resp)
}

async fn get_resp_html(pool: Pool<Postgres>) -> Result<String> {
    let sql = get_sql(pool).await?;
    let users = get_users(sql);

    let mut resp_s = String::with_capacity(80_000);

    _ = write!(resp_s, "<style> .normal {{background-color: silver;}} .highlight {{background-color: grey;}} </style><body><table>");
    for (i, user) in users.borrow().iter().enumerate() {
        if i % 25 == 0 {
            _ = write!(
                resp_s,
                "<tr><th>UID</th><th>First Name</th><th>Last Name</th></tr>"
            );
        }

        let class = if i % 5 == 0 { "highlight" } else { "normal" };
        _ = write!(
            resp_s,
            "<tr class=\"{}\"><td>{}</td><td>{}</td><td>{}</td></tr>",
            class, user.uid, user.first_name, user.last_name
        );
    }
    _ = write!(resp_s, "</table></body>");

    Ok(resp_s)
}

pub async fn users_json(State(pool): State<PgPool>) -> impl IntoResponse {
    let users = if let Ok(users) = get_resp_json(pool).await {
        users
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "application/json")], users)
}

pub async fn users_json_manual(State(pool): State<PgPool>) -> impl IntoResponse {
    let users = if let Ok(users) = get_resp_json_manual(pool).await {
        users
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "application/json")], users)
}

pub async fn users_html(State(pool): State<PgPool>) -> impl IntoResponse {
    let users = if let Ok(u) = get_resp_html(pool).await {
        u
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "text/html")], users)
}
