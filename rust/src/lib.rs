#![allow(dead_code)]
#![allow(clippy::unnecessary_unwrap)]

pub mod file_handle;
pub mod utils;

use std::fmt::Write;

use anyhow::Result;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Serialize;

use tokio_postgres::Row;
use utils::tech_emp;

type ConnectionPool = bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>;

macro_rules! get_sql {
    ($client:ident) => {
        $client.query("SELECT * from users", &[]).await
    };
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

impl ToUser for Row {
    #[inline(always)]
    fn user(&self) -> User {
        User {
            uid: self.get(0),
            fname: self.get(1),
            lname: self.get(2),
        }
    }
}

#[inline(always)]
fn get_users(rows: &[Row]) -> Vec<User> {
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

#[inline(always)]
fn get_resp_json(db_res: Vec<Row>) -> Result<Vec<u8>> {
    let mut resp = Vec::with_capacity(db_res.len() * 64);
    let users = get_users(&db_res);

    let writer = tech_emp::Writer(&mut resp);
    serde_json::to_writer(writer, &users).expect("no serial");

    Ok(resp)
}

pub async fn users_json(State(pool): State<ConnectionPool>) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    let db_res = get_sql!(client).unwrap();
    let users = if let Ok(users) = get_resp_json(db_res) {
        users
    } else {
        Vec::new()
    };

    ([(header::CONTENT_TYPE, "application/json")], users)
}

#[inline(always)]
fn get_resp_json_manual(sql: Vec<Row>) -> Result<String> {
    let mut resp = String::with_capacity(sql.len() * 64);

    resp.push('[');

    for row in &sql {
        row.user().fill_json_string(&mut resp);
    }

    resp.pop();
    resp.push(']');

    Ok(resp)
}

pub async fn users_json_manual(State(pool): State<ConnectionPool>) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    let sql = get_sql!(client).unwrap();
    let users = if let Ok(users) = get_resp_json_manual(sql) {
        users
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "application/json")], users)
}

pub async fn users_json_agg(State(pool): State<ConnectionPool>) -> impl IntoResponse {
    const QUERY: &str = "SELECT JSON_agg(users)::text from users";
    //const QUERY: &str = "SELECT json_strip_nulls(JSON_agg(users))::text from users";

    let client = pool.get().await.unwrap();
    let sql = client.query_one(QUERY, &[]).await.unwrap();
    let json: String = sql.get(0);

    ([(header::CONTENT_TYPE, "application/json")], json)
}

fn get_resp_html(sql: Vec<Row>) -> Result<String> {
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

pub async fn users_html(State(pool): State<ConnectionPool>) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    let sql = get_sql!(client).unwrap();
    let users = if let Ok(u) = get_resp_html(sql) {
        u
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "text/html")], users)
}
