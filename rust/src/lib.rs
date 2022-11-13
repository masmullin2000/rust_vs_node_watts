#![allow(dead_code)]
#![allow(clippy::unnecessary_unwrap)]

pub mod file_handle;
pub mod utils;

use std::fmt::Write;

use anyhow::Result;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Pool, Postgres, Row};

use utils::tech_emp;

#[inline(always)]
async fn get_sql(pool: Pool<Postgres>) -> Result<Vec<PgRow>> {
    let db_res = sqlx::query("SELECT * from users").fetch_all(&pool).await?;
    Ok(db_res)
}

#[derive(Serialize)]
struct User<'a> {
    uid: i32,
    fname: &'a str,
    lname: &'a str,
}

trait ToUser {
    fn user(&self) -> User;
}

impl ToUser for PgRow {
    #[inline(always)]
    fn user(&self) -> User {
        User {
            uid: self.get_unchecked(0),
            fname: self.get_unchecked(1),
            lname: self.get_unchecked(2),
        }
    }
}

#[inline(always)]
fn get_users(rows: &[PgRow]) -> Vec<User> {
    let mut x = Vec::with_capacity(rows.len());

    for i in rows {
        let u = i.user();
        x.push(u);
    }

    x
}

impl<'a> User<'a> {
    #[inline(always)]
    fn fill_json_string(&self, s: &mut String) {
        _ = write!(
            s,
            "{{\"uid\":{},\"fname\":\"{}\",\"lname\":\"{}\"}},",
            self.uid, self.fname, self.lname
        );
    }
}

fn get_resp_json(db_res: Vec<PgRow>) -> Result<Vec<u8>> {
    let mut resp = Vec::with_capacity(65_535);
    let users = get_users(&db_res);

    let writer = tech_emp::Writer(&mut resp);
    serde_json::to_writer(writer, &users).expect("no serial");

    Ok(resp)
}

pub async fn users_json(State(pool): State<PgPool>) -> impl IntoResponse {
    let db_res = get_sql(pool).await.unwrap();
    let users = if let Ok(users) = get_resp_json(db_res) {
        users
    } else {
        Vec::new()
    };

    ([(header::CONTENT_TYPE, "application/json")], users)
}

fn get_resp_json_manual(sql: Vec<PgRow>) -> Result<String> {
    let mut resp = String::with_capacity(65_535);

    resp.push('[');

    for row in &sql {
        row.user().fill_json_string(&mut resp);
    }

    resp.pop();
    resp.push(']');

    Ok(resp)
}

pub async fn users_json_manual(State(pool): State<PgPool>) -> impl IntoResponse {
    let db_res = get_sql(pool).await.unwrap();
    let users = if let Ok(users) = get_resp_json_manual(db_res) {
        users
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "application/json")], users)
}

fn get_resp_html(sql: Vec<PgRow>) -> Result<String> {
    let mut resp_s = String::with_capacity(80_000);

    _ = write!(resp_s, "<style> .normal {{background-color: silver;}} .highlight {{background-color: grey;}} </style><body><table>");
    for (i, row) in sql.iter().enumerate() {
        let user = row.user();
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
            class, user.uid, user.fname, user.lname
        );
    }
    _ = write!(resp_s, "</table></body>");

    Ok(resp_s)
}

pub async fn users_json_agg(State(pool): State<PgPool>) -> impl IntoResponse {
    const QUERY: &str = "SELECT JSON_agg(users) from users";
    //const QUERY: &str = "SELECT json_strip_nulls(JSON_agg(users)) from users";

    let res = sqlx::query(QUERY).fetch_one(&pool).await.unwrap();

    let mut resp = Vec::with_capacity(65_535);
    let mut writer = tech_emp::Writer(&mut resp);
    let users: &sqlx::types::JsonRawValue = res.get_unchecked(0);
    _ = std::io::Write::write(&mut writer, users.get().as_bytes());

    ([(header::CONTENT_TYPE, "application/json")], resp)
}

pub async fn users_html(State(pool): State<PgPool>) -> impl IntoResponse {
    let sql = get_sql(pool).await.unwrap();
    let users = if let Ok(u) = get_resp_html(sql) {
        u
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "text/html")], users)
}
