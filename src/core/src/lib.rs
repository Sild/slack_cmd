mod default_help_handler;
mod handler;
mod listener;
mod slack_cli;
mod slack_msg;
mod state;

pub mod utils;
pub use crate::handler::{ArcMsgHandler, MsgHandler, ALL_CHANNELS};
pub use crate::slack_msg::{SlackMsg, SlackMsgEv, SlackMsgHist};
pub use crate::state::BotState;

pub async fn run<I>(oauth_token: &str, socket_token: &str, msg_handlers: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = ArcMsgHandler>,
{
    let slack_cli = std::sync::Arc::new(slack_cli::SlackCliImpl::new(oauth_token)?);
    let state = BotState::new(slack_cli, msg_handlers).await?;
    let listener = listener::Listener::new(socket_token.into(), state);
    listener.serve().await
}
