mod handler;
mod listener;
mod slack_cli;
mod state;

mod default_help_handler;
pub mod utils;

pub use crate::handler::{ArcMsgHandler, MsgHandler, ALL_CHANNELS};
pub use crate::state::BotState;
pub use slack_morphism::events::SlackMessageEvent as SlackMsgEv;
use std::sync::Arc;

pub async fn run<I>(oauth_token: &str, socket_token: &str, msg_handlers: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = ArcMsgHandler>,
{
    let slack_cli = Arc::new(slack_cli::SlackCliImpl::new(oauth_token)?);
    let state = state::BotState::new(slack_cli, msg_handlers).await?;
    let listener = listener::Listener::new(socket_token.into(), state);
    listener.serve().await
}
