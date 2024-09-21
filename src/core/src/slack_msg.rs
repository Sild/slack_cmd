pub use slack_morphism::events::SlackMessageEvent as SlackMsgEv;
pub use slack_morphism::SlackHistoryMessage as SlackMsgHist;
use slack_morphism::{SlackMessageContent, SlackMessageOrigin};

pub trait SlackMsg {
    fn content(&self) -> Option<&SlackMessageContent>;
    fn origin(&self) -> &SlackMessageOrigin;
}

impl SlackMsg for SlackMsgEv {
    fn content(&self) -> Option<&SlackMessageContent> {
        self.content.as_ref()
    }
    fn origin(&self) -> &SlackMessageOrigin {
        &self.origin
    }
}

impl SlackMsg for SlackMsgHist {
    fn content(&self) -> Option<&SlackMessageContent> {
        Some(&self.content)
    }
    fn origin(&self) -> &SlackMessageOrigin {
        &self.origin
    }
}
