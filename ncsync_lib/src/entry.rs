use crate::setting::ExcludeList;
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Etag {
    etag: String,
}

impl Etag {
    pub fn get(&self) -> &str {
        &self.etag
    }

    pub fn set(&mut self, etag: &str) {
        self.etag = etag.replace("\"", "").to_string();
    }

    pub fn new(etag: &str) -> Self {
        Self {
            etag: etag.replace("\"", "").to_string(),
        }
    }
}

impl Display for Etag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.etag)
    }
}

#[derive(Debug)]
pub enum EntryType {
    File { etag: Option<Etag> },
    Dir { children: HashMap<PathBuf, Entry> },
}

impl EntryType {
    pub fn new_file(etag: Option<Etag>) -> Self {
        Self::File { etag }
    }

    pub fn new_dir() -> Self {
        Self::Dir {
            children: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub entry_type: EntryType,
    pub last_modified: DateTime<Local>,
    pub size: usize,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.path.display(),
            if self.is_dir() { "/" } else { "" }
        )
    }
}

impl Entry {
    pub fn new(
        path: PathBuf,
        entry_type: EntryType,
        last_modified: DateTime<Local>,
        size: usize,
    ) -> Self {
        Self {
            path,
            entry_type,
            last_modified,
            size,
        }
    }

    pub fn get_name(&self) -> String {
        let res = self
            .path
            .file_name()
            .unwrap_or_else(|| "".as_ref())
            .to_str()
            .unwrap_or_else(|| "");
        format!("{}{}", res, if self.is_dir() { "/" } else { "" })
    }

    pub fn get_info_str(&self) -> String {
        format!(
            "{} ({}B {})",
            self.get_name(),
            self.size,
            self.last_modified.format("%Y-%m-%d %H:%M:%S")
        )
    }

    pub fn is_dir(&self) -> bool {
        match self.entry_type {
            EntryType::Dir { .. } => true,
            _ => false,
        }
    }

    pub fn is_file(&self) -> bool {
        match self.entry_type {
            EntryType::File { .. } => true,
            _ => false,
        }
    }

    pub fn is_exclude_target(&self, exclude_list: &ExcludeList) -> bool {
        !exclude_list.judge(&self.path)
    }

    pub fn get_tree(&self, exclude_list: &ExcludeList, verbose: bool) -> String {
        let mut res = String::new();

        self.tree_rec(&mut res, "", exclude_list, verbose);

        res
    }

    fn tree_rec(&self, tree: &mut String, indent: &str, exclude_list: &ExcludeList, verbose: bool) {
        let s = format!(
            "{} {}\n",
            if verbose {
                self.get_info_str()
            } else {
                self.get_name()
            },
            if self.is_exclude_target(exclude_list) {
                "[EXCLUDE]"
            } else {
                ""
            }
        );
        tree.push_str(s.as_str());

        let children = match self {
            Entry {
                entry_type: EntryType::Dir { children },
                ..
            } => children,
            _ => return,
        };

        let mut ch_iter = children.values().peekable();
        while let Some(c) = ch_iter.next() {
            let is_not_last = ch_iter.peek().is_some();
            tree.push_str(
                format!(
                    "{}{}",
                    indent,
                    if is_not_last {
                        "├── "
                    } else {
                        "└── "
                    }
                )
                .as_str(),
            );
            c.tree_rec(
                tree,
                format!("{}{}   ", indent, if is_not_last { "│" } else { " " }).as_str(),
                exclude_list,
                verbose,
            );
        }
    }
}
