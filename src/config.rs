use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MoliConfig {
    pub api_key: String,
    pub api_secret: String,
    pub name: String,
    pub reply_times: u8,
    pub do_quote_reply: bool,
    pub do_print_results_on_console: bool,
    pub default_reply: Vec<String>,
    pub timeout_reply: Vec<String>,
}

impl Default for MoliConfig {
    fn default() -> Self {
        Self {
            api_key: Default::default(),
            api_secret: Default::default(),
            name: String::from("亚托莉"),
            reply_times: 0,
            do_quote_reply: false,
            do_print_results_on_console: false,
            default_reply: vec![
                "？".into(),
                "怎么".into(),
                "怎么了".into(),
                "什么？".into(),
                "在".into(),
                "嗯？".into(),
            ],
            timeout_reply: vec![
                "没事我就溜了".into(),
                "emmmmm".into(),
                "......".into(),
                "溜了".into(),
                "？".into(),
            ],
        }
    }
}
