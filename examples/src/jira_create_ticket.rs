use slack_cmd_core::ALL_CHANNELS;
use slack_cmd_handlers::JiraHandler;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let oauth_token = env::var("SLACK_CMD_OAUTH_TOKEN")?;
    let socket_token = env::var("SLACK_CMD_SOCKET_TOKEN")?;
    let jira_handler = JiraHandler::new("123", "345", ALL_CHANNELS.iter().cloned());
    slack_cmd_core::run(&oauth_token, &socket_token, [jira_handler]).await?;
    Ok(())
}
