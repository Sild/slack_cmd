use anyhow::Result;
use async_trait::async_trait;
use slack_cmd_core::utils::extract_channel_thread;
use slack_cmd_core::{ArcMsgHandler, BotState, MsgHandler, SlackMsgEv, ALL_CHANNELS};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

pub struct InfoHandler {}

#[async_trait]
impl MsgHandler for InfoHandler {
    fn name(&self) -> &str {
        "info"
    }
    fn description(&self) -> &str {
        "Print bot info"
    }
    fn supported_channels(&self) -> &'static HashSet<String> {
        ALL_CHANNELS.deref()
    }
    async fn handle(&self, _: &[String], msg_ev: &SlackMsgEv, bot_state: &BotState) -> Result<()> {
        let (channel, thread) = extract_channel_thread(msg_ev)?;

        let uptime = bot_state.start_time.elapsed();

        let known_channels =
            bot_state.known_channels.iter().map(|item| item.value().clone()).collect::<Vec<_>>().join("\n");

        let response = format!(
            "Bot info:\n\
        - Bot user_id: {}\n\
        - Bot user_name: {}\n\
        - Uptime: {:?}\n\
        - Known channels:\n{}",
            bot_state.bot_info.user_id.as_ref().unwrap_or(&"N/A".to_string()),
            bot_state.bot_info.name,
            uptime,
            known_channels,
        );
        bot_state.slack_cli.send_reply(&channel, &thread, &response).await
    }
}

impl InfoHandler {
    pub fn new() -> ArcMsgHandler {
        Arc::new(Self {})
    }
}
