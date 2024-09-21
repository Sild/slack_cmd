use crate::handler::ALL_CHANNELS_MARKER;
use crate::utils::extract_channel_thread;
use crate::{ArcMsgHandler, BotState, SlackMsgEv};
use std::collections::HashMap;

pub(crate) struct DefaultHelpHandler {
    // channel_name -> [handler_help_info]
    channels_help_info: HashMap<String, Vec<String>>,
    all_channels_help_info: Vec<String>,
}

impl DefaultHelpHandler {
    pub fn new(handlers: &[ArcMsgHandler]) -> Self {
        let mut channels_help_info = HashMap::new();
        let mut all_channels_help_info = Vec::new();

        for handler in handlers {
            if handler.supported_channels().contains(ALL_CHANNELS_MARKER) {
                all_channels_help_info.push(format!("`{}`: {}", handler.name(), handler.description()));
                all_channels_help_info.sort();
                continue;
            }

            for channel in handler.supported_channels() {
                let channel_help_info = channels_help_info.entry(channel.clone()).or_insert(Vec::new());
                channel_help_info.push(format!("`{}`: {}", handler.name(), handler.description()));
                channel_help_info.sort();
            }
        }

        Self {
            channels_help_info,
            all_channels_help_info,
        }
    }

    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "Prints this help message"
    }

    pub(crate) async fn handle(&self, handler_name: &str, msg_ev: &SlackMsgEv, state: &BotState) -> anyhow::Result<()> {
        let (channel, thread) = extract_channel_thread(msg_ev)?;
        let mut all_info = self.all_channels_help_info.clone();

        if let Some(channel_name) = state.known_channels.get(&channel) {
            if let Some(ch_info) = self.channels_help_info.get(channel_name.value()) {
                all_info.extend(ch_info.clone());
            };
        }

        let all_info_msg = match all_info.is_empty() {
            true => "".to_string(),
            false => format!("\n• {}", all_info.join("\n• ")),
        };
        let unknown_command_msg = if !handler_name.is_empty() && handler_name != self.name() {
            format!("Unknown command: `{}`\n", handler_name)
        } else {
            String::new()
        };
        let msg = format!(
            "{}Available commands:\n• `{}`: {}{}",
            unknown_command_msg,
            self.name(),
            self.description(),
            all_info_msg
        );
        state.slack_cli.send_reply(&channel, &thread, &msg).await
    }
}
