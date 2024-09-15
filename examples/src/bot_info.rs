use slack_cmd_handlers::InfoHandler;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let oauth_token = env::var("SLACK_CMD_OAUTH_TOKEN")?;
    let socket_token = env::var("SLACK_CMD_SOCKET_TOKEN")?;

    let info_handler = InfoHandler::new();
    slack_cmd_core::run(&oauth_token, &socket_token, [info_handler]).await?;
    Ok(())
}
