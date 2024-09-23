use crate::state::BotState;
use anyhow::Result;

use crate::utils::{extract_channel_thread, extract_msg_body};
use crate::{ArcMsgHandler, SlackMsgEv};
use slack_morphism::prelude::{
    HttpStatusCode, SlackClientEventsListenerEnvironment, SlackClientEventsUserState, SlackClientHyperConnector,
    SlackEventCallbackBody, SlackHyperClient, SlackPushEventCallback,
};
use slack_morphism::{
    SlackApiToken, SlackChannelId, SlackClient, SlackClientSocketModeConfig, SlackClientSocketModeListener,
    SlackSocketModeListenerCallbacks, SlackTs,
};
use std::sync::Arc;

pub(crate) struct Listener {
    socket_token: String,
    state: Arc<BotState>,
}

impl Listener {
    pub fn new(socket_token: String, state: BotState) -> Self {
        Self {
            socket_token,
            state: Arc::new(state),
        }
    }

    pub async fn serve(&self) -> Result<()> {
        if log::max_level() >= log::Level::Debug {
            let subscriber = tracing_subscriber::fmt().with_env_filter("slack_morphism=debug").finish();
            tracing::subscriber::set_global_default(subscriber)?;
        }

        let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
            // .with_command_events(commands_dispatcher)
            // .with_interaction_events(interactions_dispatcher)
            .with_push_events(push_events_dispatcher);

        let slack_cli = Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));
        let listener_env = Arc::new(
            SlackClientEventsListenerEnvironment::new(slack_cli)
                .with_error_handler(error_handler)
                .with_user_state(self.state.clone()),
        );

        let listener = SlackClientSocketModeListener::new(
            &SlackClientSocketModeConfig::new(),
            listener_env.clone(),
            socket_mode_callbacks,
        );

        listener.listen_for(&SlackApiToken::new(self.socket_token.clone().into())).await?;

        let exit_code = listener.serve().await;
        if exit_code != 0 {
            log::error!("Listener exited with non-zero code={}", exit_code);
        }
        Ok(())
    }
}

async fn push_events_dispatcher(
    event: SlackPushEventCallback,
    _client: Arc<SlackHyperClient>,
    state: SlackClientEventsUserState,
) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
    // process only messages here
    let message = match &event.event {
        SlackEventCallbackBody::Message(event) if event.subtype.is_none() => event.clone(),
        _ => return Ok(()),
    };
    let context_lock = state.read().await;
    let bot_state = match context_lock.get_user_state::<Arc<BotState>>() {
        Some(state) => state.clone(),
        None => {
            log::error!("Bot state is missing");
            return Ok(());
        }
    };

    let msg_body = extract_msg_body(&message)?;

    // ignore non-bot messages
    // TODO implement free_reply handler for the other cases
    if !msg_body.starts_with(&bot_state.bot_marker) {
        log::trace!("event was ignored as non-related to the bot");
        return Ok(());
    }

    log::debug!("got new push event: {:?}", &event);

    let msg_body = msg_body.strip_prefix(&bot_state.bot_marker).unwrap().trim();
    let (channel_id, thread_ts) = extract_channel_thread(&message)?;
    let handler_name = msg_body.split(' ').next().unwrap_or("help").to_string();
    let args = match shlex::split(msg_body) {
        Some(args) => args,
        None => {
            let err_msg = "Fail to parse arguments: Invalid quoting";
            return Ok(bot_state.slack_cli.send_reply(&channel_id, &thread_ts, err_msg).await?);
        }
    };

    if let Some(handler) = bot_state.get_msg_handler(&channel_id, &handler_name) {
        tokio::spawn(async move { execute_handler(handler, args, message, bot_state, channel_id, thread_ts).await });
    } else {
        tokio::spawn(async move {
            if let Err(err) = bot_state.help_handler.handle(&handler_name, &message, &bot_state).await {
                log::error!("Failed to send error message to slack: {:#?}", err);
            }
        });
    }
    Ok(())
}

async fn execute_handler(
    handler: ArcMsgHandler,
    args: Vec<String>,
    msg_ev: SlackMsgEv,
    bot_state: Arc<BotState>,
    channel_id: SlackChannelId,
    thread_ts: SlackTs,
) {
    match handler.handle(&args, &msg_ev, &bot_state).await {
        Ok(_) => log::debug!("handler {} finished successfully", handler.name()),
        Err(err) => {
            log::error!("handler failed with error: {:#?}", err);
            let error_slack_msg = "Error occurred during handling. Check logs for details.";
            if let Err(err) = bot_state.slack_cli.send_reply(&channel_id, &thread_ts, error_slack_msg).await {
                log::error!("Failed to send error message to slack: {:#?}", err);
            }
        }
    }
}

fn error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _: Arc<SlackHyperClient>,
    _: SlackClientEventsUserState,
) -> HttpStatusCode {
    log::error!("{:#?}", err);
    HttpStatusCode::OK
}

// inspired by https://github.com/abdolence/slack-morphism-rust/blob/master/examples/socket_mode.rs

// async fn interactions_dispatcher(
//     event: SlackInteractionEvent,
//     _client: Arc<SlackHyperClient>,
//     _states: SlackClientEventsUserState,
// ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     log::trace!("got new interaction event: {:#?}", event);
//     Ok(())
// }

// async fn commands_dispatcher(
//     event: SlackCommandEvent,
//     client: Arc<SlackHyperClient>,
//     _states: SlackClientEventsUserState,
// ) -> std::result::Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
//     log::trace!("got new command: {:#?}", event);
//
//     let token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_TOKEN")?.into();
//     let token = SlackApiToken::new(token_value);
//
//     // Sessions are lightweight and basically just a reference to client and token
//     let session = client.open_session(&token);
//
//     session.api_test(&SlackApiTestRequest::new().with_foo("Test".into())).await?;
//
//     let user_info_resp = session.users_info(&SlackApiUsersInfoRequest::new(event.user_id.clone())).await?;
//
//     println!("{:#?}", user_info_resp);
//
//     Ok(SlackCommandEventResponse::new(SlackMessageContent::new().with_text(
//         format!("Working on it: {:#?}", user_info_resp.user.team_id),
//     )))
// }
