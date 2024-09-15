use crate::default_help_handler::DefaultHelpHandler;
use crate::handler::{ArcMsgHandler, ALL_CHANNELS_MARKER};
use crate::slack_cli::SlackCli;
use anyhow::{bail, Result};
use dashmap::DashMap;
use slack_morphism::{SlackBotInfo, SlackChannelId};
use std::collections::HashMap;
use std::sync::Arc;

pub struct BotState {
    pub bot_marker: String,
    pub bot_info: SlackBotInfo,
    pub slack_cli: Arc<dyn SlackCli>,
    // TODO update this map if new bot was added to new channel
    pub known_channels: DashMap<SlackChannelId, String>,
    pub known_channels_rev: DashMap<String, SlackChannelId>,
    pub start_time: std::time::Instant,
    pub(crate) help_handler: DefaultHelpHandler,
    handlers_index: HandlerIndex,
}

impl BotState {
    pub(crate) async fn new<I>(slack_cli: Arc<dyn SlackCli>, handlers: I) -> Result<Self>
    where
        I: IntoIterator<Item = ArcMsgHandler>,
    {
        let bot_info = slack_cli.get_bot_info().await?;

        let known_channels = slack_cli.get_known_channels().await?.into_iter().collect::<DashMap<_, _>>();
        let known_channels_rev = known_channels.iter().map(|item| (item.value().clone(), item.key().clone())).collect();

        let bot_marker = match &bot_info.user_id {
            Some(user_id) => format!("<@{}>", user_id),
            None => bail!("Bot user_id is empty"),
        };

        let handlers = handlers.into_iter().collect::<Vec<_>>();
        let help_handler = DefaultHelpHandler::new(&handlers);
        let handlers_index = HandlerIndex::new(handlers);

        let state = Self {
            bot_marker,
            bot_info,
            slack_cli,
            known_channels,
            known_channels_rev,
            start_time: std::time::Instant::now(),
            help_handler,
            handlers_index,
        };
        Ok(state)
    }

    pub(crate) fn get_msg_handler(&self, channel_id: &SlackChannelId, handler_name: &str) -> Option<ArcMsgHandler> {
        let channel_name = match self.known_channels.get(channel_id) {
            Some(name) => name.value().clone(),
            None => {
                log::error!("channel_name not found for channel_id: {channel_id}");
                return None;
            }
        };
        self.handlers_index.get(&channel_name, handler_name)
    }
}

struct HandlerIndex {
    // channel_name -> handler_name -> handler
    channel_index: HashMap<String, HashMap<String, ArcMsgHandler>>,
    all_channels: HashMap<String, ArcMsgHandler>,
}

impl HandlerIndex {
    fn new<I>(handlers: I) -> Self
    where
        I: IntoIterator<Item = ArcMsgHandler>,
    {
        let mut channel_index = HashMap::new();
        let mut all_channels = HashMap::new();

        for handler in handlers.into_iter() {
            if handler.supported_channels().contains(ALL_CHANNELS_MARKER) {
                log::info!("handler='{}': register for all channels", handler.name());
                all_channels.insert(handler.name().to_string(), handler);
                continue;
            }
            for ch_name in handler.supported_channels().iter() {
                log::info!("handler='{}': register for channel_name='{ch_name}'", handler.name());
                let entry: &mut HashMap<String, ArcMsgHandler> = channel_index.entry(ch_name.clone()).or_default();
                entry.insert(handler.name().to_string(), handler.clone());
            }
        }

        Self {
            channel_index,
            all_channels,
        }
    }

    fn get(&self, channel_name: &str, handler_name: &str) -> Option<ArcMsgHandler> {
        if let Some(handlers) = self.channel_index.get(channel_name) {
            return handlers.get(handler_name).cloned();
        }
        self.all_channels.get(handler_name).cloned()
    }
}
