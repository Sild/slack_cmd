use anyhow::{bail, Result};
use async_trait::async_trait;
use clap::Parser;
use serde_json::{json, Value};
use slack_cmd_core::utils::{extract_channel_thread, extract_msg_body};
use slack_cmd_core::{ArcMsgHandler, BotState, MsgHandler, SlackMsgEv};
use std::cmp::min;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Parser, Clone)]
#[command(name = "jira", about = "Create jira ticket")]
struct JiraHandlerArgs {
    #[arg(short, long)]
    project: String,
    #[arg(short, long)]
    title: Option<String>,
    #[arg(short, long)]
    description: Option<String>,
}

#[allow(unused)]
pub struct JiraHandler {
    host: String,
    user: String,
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
    pub fn make<I>(jira_host: &str, user_email: &str, user_token: &str, supported_channels: I) -> ArcMsgHandler
    where
        I: IntoIterator<Item = String>,
    {
        let jira_host = jira_host.trim_end_matches('/');
        Arc::new(Self {
            host: jira_host.into(),
            user: user_email.into(),
            token: user_token.into(),
            supported_channels: supported_channels.into_iter().collect(),
        })
    }

    async fn handle_create(&self, args: &JiraHandlerArgs, msg_ev: &SlackMsgEv, bot_state: &BotState) -> Result<()> {
        let (channel, thread_ts) = extract_channel_thread(msg_ev)?;

        let mut args = args.clone();

        let root_msg = bot_state.slack_cli.get_msg(&channel, &thread_ts).await?.unwrap();
        let root_body = extract_msg_body(&root_msg)?;
        let root_body = root_body.strip_prefix(&bot_state.bot_marker).unwrap_or(&root_body);

        if args.title.is_none() {
            let len = min(root_body.len(), 50);
            args.title = Some(format!("slack: {}", &root_body[..len]));
        }

        if args.description.is_none() {
            let description = format!(
                "\
                Slack message:\n\
                {root_body}\n\n"
            );
            args.description = Some(description);
        }
        let slack_msg_link = bot_state.slack_cli.get_permalink(&channel, &thread_ts).await?;

        let issue_url = self.create_issue(&args, &slack_msg_link).await?;
        let msg = format!("Issue created: {issue_url}");
        bot_state.slack_cli.send_reply(&channel, &thread_ts, &msg).await
    }

    async fn create_issue(&self, args: &JiraHandlerArgs, slack_msg_link: &str) -> Result<String> {
        let url = format!("{}/rest/api/3/issue", self.host);
        let empty_description = String::from("No description provided");
        let body = json!({
            "fields": {
                "project": {
                    "key": args.project.to_uppercase()
                },
                "summary": args.title,
                "description": {
                "content": [
                    {
                      "content": [
                        {
                          "text": args.description.as_ref().unwrap_or(&empty_description),
                          "type": "text"
                        },
                        {
                            "type": "text",
                            "text": "[Slack message link]",
                            "marks": [
                            {
                                "type": "link",
                                "attrs": {
                                    "href": slack_msg_link
                                }
                            }
                            ]
                        }
                      ],
                      "type": "paragraph"
                    }
                ],
              "type": "doc",
              "version": 1
            },
                "issuetype": {
                    "name": "Task"
                },
            },
            "update": {},
        });
        log::debug!("creating jira issue: url={}, body={:?}", url, body.to_string());

        let rsp =
            reqwest::Client::new().post(&url).json(&body).basic_auth(&self.user, Some(&self.token)).send().await?;

        let status = rsp.status();
        let response: Value = serde_json::from_str(&rsp.text().await?)?;
        log::debug!("jira issue created: status={}, response={}", status, response.to_string());
        if status.is_success() {
            let issue_key = response["key"].as_str().unwrap();
            let issue_url = format!("{}/browse/{}", self.host, issue_key);
            Ok(issue_url)
        } else {
            bail!("Jira API call error: status: {}, msg: {}", status, response.to_string());
        }
    }
}
