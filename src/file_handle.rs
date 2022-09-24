use std::{fs::read_dir, fmt::Write, path::PathBuf};

use anyhow::Result;
use axum::response::{IntoResponse, Html};

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

