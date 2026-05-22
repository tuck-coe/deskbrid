use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_files(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Files
        "files.watch" => Action::FilesWatch {
            path: raw["path"].as_str().unwrap_or("").into(),
            recursive: raw["recursive"].as_bool().unwrap_or(true),
            patterns: raw["patterns"].as_array().map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
        },
        "files.unwatch" => Action::FilesUnwatch {
            path: raw["path"].as_str().unwrap_or("").into(),
        },
        "files.search" => Action::FilesSearch {
            pattern: raw["pattern"].as_str().unwrap_or("").into(),
            root: raw["root"].as_str().map(String::from),
            max_results: raw["max_results"].as_u64().unwrap_or(50) as u32,
        },
        "files.read" => Action::FilesRead {
            path: raw["path"].as_str().unwrap_or("").into(),
            offset: raw["offset"].as_u64(),
            limit: raw["limit"].as_u64(),
        },
        "files.write" => Action::FilesWrite {
            path: raw["path"].as_str().unwrap_or("").into(),
            content: raw["content"].as_str().unwrap_or("").into(),
            append: raw["append"].as_bool().unwrap_or(false),
        },
        "files.copy" => Action::FilesCopy {
            source: raw["source"].as_str().unwrap_or("").into(),
            destination: raw["destination"].as_str().unwrap_or("").into(),
        },
        "files.move" => Action::FilesMove {
            source: raw["source"].as_str().unwrap_or("").into(),
            destination: raw["destination"].as_str().unwrap_or("").into(),
        },
        "files.delete" => Action::FilesDelete {
            path: raw["path"].as_str().unwrap_or("").into(),
            recursive: raw["recursive"].as_bool().unwrap_or(false),
        },
        "files.mkdir" => Action::FilesMkdir {
            path: raw["path"].as_str().unwrap_or("").into(),
            parents: raw["parents"].as_bool().unwrap_or(true),
        },
        "files.list" => Action::FilesList {
            path: raw["path"].as_str().unwrap_or(".").into(),
        },
        _ => anyhow::bail!("unknown files type: {type_str}"),
    })
}
