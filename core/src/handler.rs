use crate::state::BotState;
use anyhow::Result;
use async_trait::async_trait;
use slack_morphism::events::SlackMessageEvent as SlackMsgEv;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

#[async_trait]
pub trait MsgHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn supported_channels(&self) -> &HashSet<String>;
    async fn handle(&self, args: &[String], msg_ev: &SlackMsgEv, state: &BotState) -> Result<()>;
}
pub type ArcMsgHandler = Arc<dyn MsgHandler>;
pub const ALL_CHANNELS_MARKER: &str = "*";
pub static ALL_CHANNELS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert(ALL_CHANNELS_MARKER.to_string());
    set
});
