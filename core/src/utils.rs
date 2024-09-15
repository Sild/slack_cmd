use crate::SlackMsgEv;
use anyhow::{anyhow, Result};
use slack_morphism::{SlackChannelId, SlackTs};

pub fn extract_msg_body(msg: &SlackMsgEv) -> Result<String> {
    let content = msg.content.as_ref().ok_or_else(|| anyhow!("content is None"))?;
    let text = content.text.as_ref().ok_or_else(|| anyhow!("text is None"))?;
    Ok(text.clone())
}
pub fn extract_channel_id(msg: &SlackMsgEv) -> Result<SlackChannelId> {
    msg.origin.channel.as_ref().ok_or_else(|| anyhow!("Channel not found in msg")).cloned()
}

pub fn extract_thread_ts(msg: &SlackMsgEv) -> SlackTs {
    msg.origin.thread_ts.as_ref().unwrap_or(&msg.origin.ts).clone()
}

pub fn extract_channel_thread(msg: &SlackMsgEv) -> Result<(SlackChannelId, SlackTs)> {
    let channel_id = extract_channel_id(msg)?;
    let thread_ts = extract_thread_ts(msg);
    Ok((channel_id.clone(), thread_ts))
}
