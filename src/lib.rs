#![allow(dead_code)]
#![allow(clippy::unnecessary_unwrap)]

use core::slice;
use std::fs::read_dir;
use std::path::PathBuf;
use std::{cell::RefCell, fmt::Write};

use anyhow::Result;
use axum::http::header;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use serde::Serialize;
use sqlx::{postgres::PgRow, PgPool, Pool, Postgres, Row};

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

type sstring = smallstr::SmallString<[u8; 16]>;
#[derive(Serialize)]
struct User {
    uid: i32,
    first_name: sstring,
    last_name: sstring,
}

async fn get_sql(pool: Pool<Postgres>) -> Result<Vec<PgRow>> {
    let db_res = sqlx::query("SELECT * from users")
        .bind("uid")
        .bind("first_name")
        .bind("last_name")
        .fetch_all(&pool)
        .await?;
    Ok(db_res)
}

fn get_users<'a>(sql_rows: Vec<PgRow>) -> &'a [User] {
    thread_local! {
        static USERS: RefCell<Vec<User>> = RefCell::new(Vec::with_capacity(1000));
    }

    USERS.with(|u| {
        let users = &mut *u.borrow_mut();
        users.clear();

        sql_rows.into_iter().for_each(|row| {
            let uid: i32 = row.get(0);
            let fname: &str = row.get(1);
            let lname: &str = row.get(2);
            users.push(User {
                uid,
                first_name: sstring::from(fname),
                last_name: sstring::from(lname),
            })
        });
        //users.shrink_to(1000);

        let ptr = users.as_ptr();
        unsafe { slice::from_raw_parts(ptr, users.len()) }
    })
}

fn get_resp(sql: Vec<PgRow>) -> Vec<u8> {
    let mut resp = Vec::with_capacity(65_535);
    let users = get_users(sql);
    
    let writer = std::io::BufWriter::new(&mut resp);
    serde_json::to_writer(writer, users).expect("blah");

    resp
}

pub async fn users(State(pool): State<PgPool>) -> impl IntoResponse {
    let users = if let Ok(sql) = get_sql(pool).await {
        let resp = get_resp(sql);
        let r = unsafe { String::from_utf8_unchecked(resp) };
        r
    } else {
        "".to_string()
    };

    ([(header::CONTENT_TYPE, "application/json")], users) 
}
