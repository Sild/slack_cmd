mod cli_impl;
pub use cli_impl::SlackCliImpl;

use anyhow::Result;
use async_trait::async_trait;
use slack_morphism::{SlackBotInfo, SlackChannelId, SlackMessage, SlackTs};
use std::collections::HashMap;
use tracing::log;

#[async_trait]
pub trait SlackCli: Send + Sync {
    async fn get_thread(&self, _channel: &SlackChannelId, _msg_ts: &SlackTs) -> Result<Vec<SlackMessage>> {
        Ok(vec![])
    }
    async fn get_message(&self, _channel: &SlackChannelId, _msg_ts: &SlackTs) -> Result<()> {
        Ok(())
    }
    async fn delete_msg(&self, _channel: &SlackChannelId, _msg_ts: &SlackTs) -> Result<()> {
        Ok(())
    }
    async fn send_msg(&self, channel: &SlackChannelId, msg: &str) -> Result<()> {
        log::info!("send_msg: channel_id='{channel}', msg='{msg}'");
        Ok(())
    }
    async fn send_reply(&self, channel: &SlackChannelId, thread_ts: &SlackTs, msg: &str) -> Result<()> {
        log::info!("send_reply: channel_id='{channel}', thread_ts='{thread_ts}', msg='{msg}'");
        Ok(())
    }
    async fn get_bot_info(&self) -> Result<SlackBotInfo> {
        Ok(SlackBotInfo {
            id: Some("default_bot_id".into()),
            name: "default_bot_name".into(),
            updated: None,
            app_id: "default_app_id".into(),
            user_id: Some("default_user_id".into()),
            icons: None,
        })
    }
    async fn get_known_channels(&self) -> Result<HashMap<SlackChannelId, String>> {
        Ok(HashMap::new())
    }
}
