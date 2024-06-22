use serde::{Deserialize, Serialize};
use std::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Text {
    pub text: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SubMatch {
    #[serde(rename = "match")]
    pub submatch: Text,
    pub start: usize,
    pub end: usize,
}

//TODO: Missing fields
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Stats {
    pub matched_lines: u32,
    pub matches: u32,
    pub searches: u32,
    pub searches_with_match: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "data")]
pub enum Match {
    Begin {
        path: Text,
    },
    Match {
        path: Text,
        lines: Text,
        line_number: u32,
        absolute_offset: u32,
        submatches: Vec<SubMatch>,
    },
    End {
        path: Text,
    },
    Summary {
        stats: Stats,
    },
}
