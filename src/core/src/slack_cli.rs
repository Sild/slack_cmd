mod cli_impl;
pub use cli_impl::SlackCliImpl;

use crate::SlackMsgHist;
use anyhow::Result;
use async_trait::async_trait;
use slack_morphism::{SlackBotInfo, SlackChannelId, SlackTs};
use std::collections::HashMap;

#[async_trait]
pub trait SlackCli: Send + Sync {
    async fn send_msg(&self, channel: &SlackChannelId, msg: &str) -> Result<()> {
        self.send_msg_impl(channel, None, msg).await
    }

    async fn send_reply(&self, channel: &SlackChannelId, thread_ts: &SlackTs, msg: &str) -> Result<()> {
        self.send_msg_impl(channel, Some(thread_ts), msg).await
    }

    async fn send_msg_impl(&self, _channel: &SlackChannelId, msg_ts: Option<&SlackTs>, msg: &str) -> Result<()>;

    async fn get_thread(&self, channel: &SlackChannelId, thread_ts: &SlackTs) -> Result<Vec<SlackMsgHist>> {
        Ok(self.get_msgs_impl(channel, thread_ts, None).await?)
    }

    async fn get_msg(&self, channel: &SlackChannelId, msg_ts: &SlackTs) -> Result<Option<SlackMsgHist>> {
        Ok(self.get_msgs_impl(channel, msg_ts, Some(1)).await?.pop())
    }

    async fn get_permalink(&self, channel: &SlackChannelId, msg_ts: &SlackTs) -> Result<String>;

    async fn get_msgs_impl(
        &self,
        channel: &SlackChannelId,
        msg_ts: &SlackTs,
        limit: Option<u16>,
    ) -> Result<Vec<SlackMsgHist>>;

    async fn delete_msg(&self, _channel: &SlackChannelId, _msg_ts: &SlackTs) -> Result<()> {
        todo!()
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
    async fn get_known_channels(&self) -> Result<HashMap<SlackChannelId, String>>;
}
