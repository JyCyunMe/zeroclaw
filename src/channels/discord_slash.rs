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
        /// Respond to an interaction with a message.
        /// Must be sent within 3 seconds of receiving the interaction.
        pub const CHANNEL_MESSAGE_WITH_SOURCE: i32 = 4;

        /// ACK an interaction and edit a response later.
        /// The user sees a loading state ("Bot is thinking...").
        /// Must be sent within 3 seconds of receiving the interaction.
        /// After deferring, you have up to 15 minutes to edit the response.
        pub const DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE: i32 = 5;
    }
}

/// Discord Application Command structure for registration
///
/// Note: `type_` uses `#[serde(rename = "type")]` because `type` is a Rust reserved word,
/// but Discord API expects the JSON field name to be exactly `"type"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordCommand {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<DiscordCommandOption>>,
    #[serde(default, rename = "type")]
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
///
/// Note: `type_` uses `#[serde(rename = "type")]` because `type` is a Rust reserved word,
/// but Discord API expects the JSON field name to be exactly `"type"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordCommandOption {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
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
    #[serde(default)]
    pub member: Option<DiscordMember>,
    #[serde(default)]
    pub user: Option<DiscordUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMember {
    pub user: Option<DiscordUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub discriminator: Option<String>,
}

impl DiscordInteraction {
    pub fn user_id(&self) -> Option<&str> {
        self.member
            .as_ref()
            .and_then(|m| m.user.as_ref())
            .map(|u| u.id.as_str())
            .or_else(|| self.user.as_ref().map(|u| u.id.as_str()))
    }

    pub fn username(&self) -> Option<&str> {
        self.member
            .as_ref()
            .and_then(|m| m.user.as_ref())
            .and_then(|u| u.username.as_deref())
            .or_else(|| self.user.as_ref().and_then(|u| u.username.as_deref()))
    }
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
        DiscordCommand::new("help", "Show help information and usage guide"),
        DiscordCommand::new("commands", "List all available slash commands"),
        DiscordCommand::new("whoami", "Display your Discord user ID"),
        DiscordCommand::new("skills", "List all available skills"),
        DiscordCommand::new("skill", "Execute a specific skill by name").with_options(vec![
            DiscordCommandOption::new(
                "name",
                "Name of the skill to execute",
                types::option_type::STRING,
            )
            .required(),
            DiscordCommandOption::new(
                "input",
                "Optional input for the skill",
                types::option_type::STRING,
            ),
        ]),
        DiscordCommand::new("verbose", "Toggle verbose mode for detailed responses").with_options(
            vec![
                DiscordCommandOption::new("mode", "on or off", types::option_type::STRING)
                    .required(),
            ],
        ),
        DiscordCommand::new("models", "List available AI models"),
        DiscordCommand::new("model", "Show current AI model"),
        DiscordCommand::new("new", "Start a new conversation session"),
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
        assert_eq!(cmds.len(), 10);

        let names: Vec<&str> = cmds.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"help"));
        assert!(names.contains(&"commands"));
        assert!(names.contains(&"whoami"));
        assert!(names.contains(&"skills"));
        assert!(names.contains(&"skill"));
        assert!(names.contains(&"verbose"));
        assert!(names.contains(&"models"));
        assert!(names.contains(&"model"));
        assert!(names.contains(&"new"));
        assert!(names.contains(&"bind"));
    }
}
