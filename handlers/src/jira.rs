use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use slack_cmd_core::utils::extract_channel_thread;
use slack_cmd_core::{ArcMsgHandler, BotState, MsgHandler, SlackMsgEv};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Parser)]
#[command(name = "jira", about = "Create jira ticket")]
pub struct JiraHandlerArgs {
    project: String,
    title: String,
}

#[allow(unused)]
pub struct JiraHandler {
    host: String,
    token: String,
    supported_channels: HashSet<String>,
}

#[async_trait]
impl MsgHandler for JiraHandler {
    fn name(&self) -> &str {
        "jira"
    }

    fn description(&self) -> &str {
        "Create jira ticket"
    }

    fn supported_channels(&self) -> &HashSet<String> {
        &self.supported_channels
    }

    async fn handle(&self, args: &[String], msg_ev: &SlackMsgEv, bot_state: &BotState) -> Result<()> {
        let (channel, thread) = extract_channel_thread(msg_ev)?;

        let parsed_args = match JiraHandlerArgs::try_parse_from(args) {
            Ok(args) => args,
            Err(err) => {
                bot_state.slack_cli.send_reply(&channel, &thread, &format!("{}", err)).await?;
                return Ok(());
            }
        };
        self.handle_create(&parsed_args, msg_ev, bot_state).await
    }
}

impl JiraHandler {
    pub fn new<I>(jira_host: &str, jira_token: &str, supported_channels: I) -> ArcMsgHandler
    where
        I: IntoIterator<Item = String>,
    {
        Arc::new(Self {
            host: jira_host.into(),
            token: jira_token.into(),
            supported_channels: supported_channels.into_iter().collect(),
        })
    }

    async fn handle_create(&self, args: &JiraHandlerArgs, msg_ev: &SlackMsgEv, bot_state: &BotState) -> Result<()> {
        let (channel, thread) = extract_channel_thread(msg_ev)?;
        let msg = format!("Creating jira ticket for project: {}, title: {}", args.project, args.title);
        bot_state.slack_cli.send_reply(&channel, &thread, &msg).await
    }
}
