mod config;
mod data;

use crate::config::MoliConfig;
use crate::data::{MoliMessage, MoliResponse};
use atri_plugin::event::GroupMessageEvent;
use atri_plugin::listener::{Listener, ListenerGuard};
use atri_plugin::message::meta::Reply;
use atri_plugin::message::{MessageChainBuilder, MessageValue};
use atri_plugin::{error, info, Plugin};
use dashmap::DashSet;
use rand::Rng;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

static MOLI_REQ_URL: &str = "https://api.mlyai.com/reply";

#[atri_plugin::plugin(name = "moli_atri")]
struct MoliAtri {
    runtime: Arc<tokio::runtime::Runtime>,
    client: reqwest::Client,
    listener: Option<ListenerGuard>,
}

impl Plugin for MoliAtri {
    fn new() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap().into(),
            client: reqwest::Client::builder().build().unwrap(),
            listener: None,
        }
    }

    fn enable(&mut self) {
        let config_file = atri_plugin::env::workspace().join("config.toml");
        let conf = 'err: {
            let Ok(bytes) = std::fs::read(&config_file) else {
                break 'err None;
            };

            let Ok(config) = toml::from_slice::<MoliConfig>(&bytes) else {
                break 'err None;
            };

            Some(config)
        };

        let config = conf.unwrap_or_else(|| {
            let def = MoliConfig::default();
            let _ = std::fs::write(
                &config_file,
                toml::to_string_pretty(&def).expect("Cannot serialize"),
            );
            def
        });
        let config = Arc::new(config);
        let set = DashSet::<i64>::new();
        let set = Arc::new(set);

        let client_shared = self.client.clone();

        let rt = self.runtime.clone();

        self.listener = Some(Listener::listening_on_always(
            move |e: GroupMessageEvent| {
                let config = config.clone();
                let client = client_shared.clone();

                let set = set.clone();
                let rt = rt.clone();
                async move {
                    let _ = rt
                        .spawn(async move {
                            if set.contains(&e.sender().id()) {
                                return;
                            }
                            let f = || {
                                let msg = e.message();
                                for elem in msg {
                                    match elem {
                                        MessageValue::At(at) if at.target == e.client().id() => {
                                            return true
                                        }
                                        MessageValue::Text(s) if s.contains(&config.name) => {
                                            return true
                                        }
                                        _ => {}
                                    }
                                }
                                false
                            };

                            if !f() {
                                return;
                            }

                            let sender = e.sender().id();
                            set.insert(sender);

                            let mut e = e;

                            if let Err(e) = handle_message(&client, &e, &config).await {
                                error!("Error on handle message {}", e);
                            }

                            let mut replied = false;
                            for _ in 0..config.reply_times {
                                e = if let Some(e) = e
                                    .next(Duration::from_secs(10), move |e| {
                                        e.sender().id() == sender
                                    })
                                    .await
                                {
                                    replied = true;
                                    e
                                } else if !replied {
                                    let mut msg = MessageChainBuilder::new();
                                    let random =
                                        rand::thread_rng().gen_range(0..config.timeout_reply.len());
                                    msg.push_str(&config.timeout_reply[random]);
                                    let _ = e.group().send_message(msg.build()).await;

                                    set.remove(&sender);
                                    return;
                                } else {
                                    set.remove(&sender);
                                    return;
                                };

                                if let Err(e) = handle_message(&client, &e, &config).await {
                                    error!("Error on handle message {}", e);
                                }
                            }

                            set.remove(&sender);
                        })
                        .await;
                }
            },
        ))
    }
}

async fn handle_message(
    client: &reqwest::Client,
    e: &GroupMessageEvent,
    config: &MoliConfig,
) -> Result<(), Box<dyn Error>> {
    let msg = MoliMessage::from(e.clone());

    let json = serde_json::to_string(&msg)?;
    let resp = client
        .post(MOLI_REQ_URL)
        .header("Api-Key", &config.api_key)
        .header("Api-Secret", &config.api_secret)
        .header("Content-Type", "application/json;charset=UTF-8")
        .body(json)
        .send()
        .await?;

    let resp: MoliResponse = serde_json::from_slice(&resp.bytes().await?)?;

    if config.do_print_results_on_console {
        info!("服务器返回数据: {:?}", resp);
    }

    if resp.code != "00000" {
        error!("出现异常: code={} message={}", resp.code, resp.message);
        return Ok(());
    }

    let mut msg = MessageChainBuilder::new();

    for dat in resp.data {
        match dat.typed {
            1 => {
                msg.push_str(&dat.content);
            }
            2 => {
                let img = String::from("https://files.molicloud.com/") + &dat.content;
                let img = client.get(img).send().await?;

                msg.push(e.group().upload_image(img.bytes().await?.to_vec()).await?);
            }
            _ => {}
        };
    }

    let message = e.message();
    if config.do_quote_reply {
        let r = Reply {
            reply_seq: message.metadata().seqs[0],
            sender: e.sender().id(),
            time: e.message().metadata().time,
            elements: message.into_iter().collect(),
        };

        msg.with_reply(r);
    }

    e.group().send_message(msg.build()).await?;

    Ok(())
}
