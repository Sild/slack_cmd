use crate::slack_cli::SlackCli;
use crate::SlackMsgHist;
use anyhow::Result;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use slack_morphism::api::{
    SlackApiBotsInfoRequest, SlackApiChatDeleteRequest, SlackApiChatGetPermalinkRequest,
    SlackApiChatPostMessageRequest, SlackApiConversationsListRequest, SlackApiConversationsRepliesRequest,
};
use slack_morphism::hyper_tokio::{SlackClientHyperConnector, SlackClientHyperHttpsConnector};
use slack_morphism::{
    SlackApiToken, SlackBotInfo, SlackChannelId, SlackClient, SlackClientSession, SlackConversationType,
    SlackMessageContent, SlackTs,
};
use std::collections::HashMap;

pub struct SlackCliImpl {
    token: SlackApiToken,
    client: SlackClient<SlackClientHyperHttpsConnector>,
}

impl SlackCliImpl {
    pub fn new(token: &str) -> Result<Self> {
        let client = SlackClient::new(SlackClientHyperConnector::new()?);
        Ok(Self {
            token: SlackApiToken::new(token.into()),
            client,
        })
    }

    // allow to get raw session for custom workflow
    pub fn get_session(&self) -> SlackClientSession<SlackClientHyperHttpsConnector> {
        self.client.open_session(&self.token)
    }
}

#[async_trait]
impl SlackCli for SlackCliImpl {
    async fn send_msg_impl(&self, channel: &SlackChannelId, thread_ts: Option<&SlackTs>, msg: &str) -> Result<()> {
        log::trace!("send_msg_impl: channel_id='{channel}', thread_ts='{:?}', msg='{msg}'", thread_ts);
        let mut req = SlackApiChatPostMessageRequest::new(
            format!("{}", channel).into(),
            SlackMessageContent::new().with_text(msg.into()),
        );
        if let Some(thread_ts) = thread_ts {
            req = req.with_thread_ts(thread_ts.clone());
        }
        match self.get_session().chat_post_message(&req).await {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::log::error!("Fail to send msg='{msg}' to channel='{channel}', err='{:?}'", err);
                Err(anyhow!(err))
            }
        }
    }

    async fn get_permalink(&self, channel: &SlackChannelId, msg_ts: &SlackTs) -> Result<String> {
        let req = SlackApiChatGetPermalinkRequest::new(channel.clone(), msg_ts.clone());
        match self.get_session().chat_get_permalink(&req).await {
            Ok(rsp) => Ok(rsp.permalink.to_string()),
            Err(err) => {
                bail!(err)
            }
        }
    }

    async fn get_msgs_impl(
        &self,
        channel: &SlackChannelId,
        ts: &SlackTs,
        limit: Option<u16>,
    ) -> Result<Vec<SlackMsgHist>> {
        let time_limits = if limit.is_some() && limit.as_ref().unwrap() == &1 {
            Some(ts.clone())
        } else {
            None
        };
        let req = SlackApiConversationsRepliesRequest {
            channel: channel.clone(),
            ts: ts.clone(),
            cursor: None,
            latest: time_limits.clone(),
            limit,
            oldest: time_limits.clone(),
            inclusive: Some(true),
        };
        Ok(self.get_session().conversations_replies(&req).await?.messages)
    }

    async fn delete_msg(&self, channel: &SlackChannelId, msg_ts: &SlackTs) -> Result<()> {
        let req = SlackApiChatDeleteRequest::new(channel.clone(), msg_ts.clone());
        match self.get_session().chat_delete(&req).await {
            Ok(_) => Ok(()),
            Err(err) => {
                log::warn!("Fail to delete msg from channel='{channel}' with ts='{msg_ts}, err='{:?}'", err);
                bail!(err)
            }
        }
    }

    async fn get_bot_info(&self) -> Result<SlackBotInfo> {
        let session = self.get_session();
        let auth_info = session.auth_test().await?;
        let rsp =
            session.bots_info(&SlackApiBotsInfoRequest::new().with_bot(auth_info.bot_id.unwrap().to_string())).await;
        match rsp {
            Ok(rsp) => Ok(rsp.bot),
            Err(err) => {
                tracing::log::error!("Fail to get bot info, err='{:?}'", err);
                bail!(err)
            }
        }
    }

    async fn get_known_channels(&self) -> Result<HashMap<SlackChannelId, String>> {
        let mut result = HashMap::new();
        let session = self.get_session();
        let mut req = SlackApiConversationsListRequest {
            cursor: None,
            limit: Some(100),
            exclude_archived: Some(true),
            types: Some(vec![SlackConversationType::Public, SlackConversationType::Private]),
        };
        loop {
            let rsp = session.conversations_list(&req).await?;
            for channel in rsp.channels {
                result.insert(channel.id.clone(), channel.name.ok_or(anyhow!("Channel name is missing"))?);
            }
            if let Some(cursor) = rsp.response_metadata.and_then(|x| x.next_cursor) {
                req.cursor = Some(cursor);
            } else {
                break;
            }
        }
        Ok(result)
    }
}
