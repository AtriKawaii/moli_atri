use atri_plugin::contact::member::Member;
use atri_plugin::event::GroupMessageEvent;
use atri_plugin::message::MessageValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MoliMessage {
    pub content: String,
    #[serde(rename = "type")]
    pub chat_type: u8,
    pub from: i64,
    #[serde(rename = "fromName")]
    pub from_name: String,
    pub to: i64,
    #[serde(rename = "toName")]
    pub to_name: String,
}

impl From<GroupMessageEvent> for MoliMessage {
    fn from(e: GroupMessageEvent) -> Self {
        let mut content = String::new();
        let msg = e.message();
        let sender = e.sender();
        let name = match sender {
            Member::Named(named) => named.card_name().to_string(),
            Member::Anonymous(_) => "匿名用户".to_string()
        };
        let group = e.group();

        for val in msg {
            match val {
                MessageValue::Text(s) => content.push_str(&s),
                MessageValue::At(at) => content.push_str(&at.display),
                MessageValue::AtAll => content.push_str("所有人"),
                MessageValue::Image(_) => content.push_str("[图片]"),
                _ => {}
            }
        }

        Self {
            content,
            chat_type: 2,
            from: e.sender().id(),
            from_name: name,
            to: group.id(),
            to_name: group.name().into()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MoliData {
    pub content: String,
    pub typed: u8,
    pub remark: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MoliResponse {
    pub code: String,
    pub message: String,
    pub plugin: Option<String>,
    pub data: Vec<MoliData>,
}