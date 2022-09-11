#![allow(dead_code)]
#![allow(clippy::unnecessary_unwrap)]

use std::fmt::Write;
use std::fs::read_dir;
use std::path::PathBuf;

use anyhow::Result;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Json,
};
use serde::Serialize;
use sqlx::{PgPool, Pool, Postgres, Row};

const FILE: &str = include_str!("../manpage");

fn walk(file_list: &mut Vec<String>, root: PathBuf) -> Result<()> {
    let dir = read_dir(root)?;

    for entry in dir {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            walk(file_list, entry.path())?;
        } else if meta.is_file() {
            if let Some(s) = entry.path().to_str() {
                let s = String::from(s);
                file_list.push(s);
            }
        }
    }

    Ok(())
}

pub async fn file_list() -> impl IntoResponse {
    let mut resp_h = String::from("<ul>");

    let mut file_list = Vec::new();
    if walk(&mut file_list, PathBuf::from("/my_tmp/mainline/")).is_ok() {
        file_list.sort_unstable();

        for file in file_list {
            _ = writeln!(resp_h, "<li>{}</li>", file);
        }
        resp_h.push_str("</ul>");
    }

    Html(resp_h)
}

pub async fn file() -> impl IntoResponse {
    Html(FILE)
}

#[derive(Serialize)]
struct User {
    uid: i32,
    first_name: String,
    last_name: String,
}

async fn get_users(pool: Pool<Postgres>) -> Result<Vec<User>> {
    let db_res = sqlx::query("SELECT * from users")
        .bind("uid")
        .bind("first_name")
        .bind("last_name")
        .fetch_all(&pool)
        .await?;

    let users: Vec<User> = db_res
        .into_iter()
        .map(|row| {
            let uid: i32 = row.get(0);
            let fname: &str = row.get(1);
            let lname: &str = row.get(2);
            User {
                uid,
                first_name: String::from(fname),
                last_name: String::from(lname),
            }
        })
        .collect();

    Ok(users)
}

pub async fn users(State(pool): State<PgPool>) -> impl IntoResponse {
    let users = get_users(pool).await.unwrap();
    Json(users)
}
