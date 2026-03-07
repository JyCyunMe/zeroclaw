//! Discord Slash Commands implementation
//!
//! This module handles Discord Application Commands (slash commands):
//! - Command registration via Discord API
//! - INTERACTION_CREATE event handling
//! - Command execution logic

use serde::{Deserialize, Serialize};

pub mod types {
    pub const CHAT_INPUT: i32 = 1;

    pub mod option_type {
        pub const STRING: i32 = 3;
        pub const INTEGER: i32 = 4;
        pub const BOOLEAN: i32 = 5;
    }

    pub mod interaction_type {
        pub const APPLICATION_COMMAND: i32 = 2;
    }

    pub mod callback_type {
        pub const CHANNEL_MESSAGE_WITH_SOURCE: i32 = 4;
    }
}

/// Discord Application Command structure for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordCommand {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<DiscordCommandOption>>,
    #[serde(default)]
    pub type_: i32,
}

impl DiscordCommand {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            options: None,
            type_: types::CHAT_INPUT,
        }
    }

    fn with_options(mut self, options: Vec<DiscordCommandOption>) -> Self {
        self.options = Some(options);
        self
    }
}

/// Discord Command Option (parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordCommandOption {
    pub name: String,
    pub description: String,
    pub type_: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

impl DiscordCommandOption {
    fn new(name: &str, description: &str, type_: i32) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            type_,
            required: None,
        }
    }

    fn required(mut self) -> Self {
        self.required = Some(true);
        self
    }
}

/// Discord Interaction structure (received when user uses slash command)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordInteraction {
    pub id: String,
    pub application_id: String,
    #[serde(rename = "type")]
    pub interaction_type: i32,
    pub data: Option<InteractionData>,
    pub channel_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionData {
    pub name: String,
    #[serde(default)]
    pub options: Vec<InteractionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOption {
    pub name: String,
    pub value: Option<serde_json::Value>,
}

/// Callback response for Discord Interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionCallback {
    #[serde(rename = "type")]
    pub callback_type: i32,
    pub data: Option<CallbackData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackData {
    pub content: String,
}

/// Generate all slash commands for ZeroClaw
pub fn zeroclaw_slash_commands() -> Vec<DiscordCommand> {
    vec![
        // /models - List available models
        DiscordCommand::new("models", "List available AI models"),
        // /model - Show current model
        DiscordCommand::new("model", "Show current AI model"),
        // /new - Start new session
        DiscordCommand::new("new", "Start a new conversation session"),
        // /bind - Bind account with pairing code
        DiscordCommand::new("bind", "Bind your Discord account using a pairing code").with_options(
            vec![DiscordCommandOption::new(
                "code",
                "6-digit pairing code from operator",
                types::option_type::STRING,
            )
            .required()],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let cmd = DiscordCommand::new("test", "Test command");
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"description\":\"Test command\""));
    }

    #[test]
    fn test_command_with_options() {
        let cmd = DiscordCommand::new("bind", "Bind account").with_options(vec![
            DiscordCommandOption::new("code", "Pairing code", types::option_type::STRING)
                .required(),
        ]);

        assert!(cmd.options.is_some());
        let opts = cmd.options.unwrap();
        assert_eq!(opts.len(), 1);
        assert_eq!(opts[0].name, "code");
        assert_eq!(opts[0].required, Some(true));
    }

    #[test]
    fn test_zeroclaw_commands() {
        let cmds = zeroclaw_slash_commands();
        assert_eq!(cmds.len(), 4);

        let names: Vec<&str> = cmds.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"models"));
        assert!(names.contains(&"model"));
        assert!(names.contains(&"new"));
        assert!(names.contains(&"bind"));
    }
}
