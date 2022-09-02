use std::path::PathBuf;

use anyhow::Result;
use axum::response::{IntoResponse, Html};
use async_recursion::async_recursion;

use std::fs::read_dir;

fn walk(file_list: &mut Vec<String>, root: PathBuf) -> Result<()> {
    let dir = read_dir(root)?;

    for entry in dir {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            _ = walk(file_list, entry.path())?;
        } else if meta.is_file() {
            let s = entry.path().to_string_lossy().to_string();
            file_list.push(s);
        }
    }

    Ok(())
}

pub async fn file_list() -> impl IntoResponse {
    let mut resp_h = String::from("<ul>");

    let mut file_list = Vec::new();
    _ = walk(&mut file_list, PathBuf::from("/code/mainline/"));

    file_list.sort_unstable();

    for file in file_list {
        resp_h.push_str(&format!("<li>{}</li>\n", file));
    }
    resp_h.push_str("</ul>");

    Html(resp_h)
}
    
