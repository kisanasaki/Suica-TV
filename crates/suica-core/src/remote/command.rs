use crate::{error::CoreError, mode::DisplayMode};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationAction {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    Home,
}

impl NavigationAction {
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Up => "navigation.up",
            Self::Down => "navigation.down",
            Self::Left => "navigation.left",
            Self::Right => "navigation.right",
            Self::Select => "navigation.select",
            Self::Back => "navigation.back",
            Self::Home => "navigation.home",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemoteAction {
    Navigation(NavigationAction),
    SwitchMode(DisplayMode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteCommand {
    pub request_id: Uuid,
    pub action: RemoteAction,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CommandDto {
    #[serde(rename = "type")]
    kind: String,
    request_id: Uuid,
    action: String,
    params: Option<ParamsDto>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsDto {
    mode: DisplayMode,
}

impl RemoteCommand {
    pub fn parse(text: &str) -> Result<Self, CoreError> {
        let dto: CommandDto = serde_json::from_str(text).map_err(|_| CoreError::InvalidMessage)?;
        if dto.kind != "remote.command" {
            return Err(CoreError::InvalidMessage);
        }
        let action = match dto.action.as_str() {
            "navigation.up" => RemoteAction::Navigation(NavigationAction::Up),
            "navigation.down" => RemoteAction::Navigation(NavigationAction::Down),
            "navigation.left" => RemoteAction::Navigation(NavigationAction::Left),
            "navigation.right" => RemoteAction::Navigation(NavigationAction::Right),
            "navigation.select" => RemoteAction::Navigation(NavigationAction::Select),
            "navigation.back" => RemoteAction::Navigation(NavigationAction::Back),
            "navigation.home" => RemoteAction::Navigation(NavigationAction::Home),
            "system.switch_mode" => {
                RemoteAction::SwitchMode(dto.params.as_ref().ok_or(CoreError::InvalidMessage)?.mode)
            }
            _ => return Err(CoreError::InvalidCommand),
        };
        if !matches!(action, RemoteAction::SwitchMode(_)) && dto.params.is_some() {
            return Err(CoreError::InvalidMessage);
        }
        Ok(Self {
            request_id: dto.request_id,
            action,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_navigation() {
        let c=RemoteCommand::parse(r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"navigation.up"}"#).unwrap();
        assert_eq!(c.action, RemoteAction::Navigation(NavigationAction::Up));
    }
    #[test]
    fn rejects_unknown() {
        assert!(matches!(
            RemoteCommand::parse(
                r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"shell.run"}"#
            ),
            Err(CoreError::InvalidCommand)
        ));
    }
    #[test]
    fn parses_switch_mode_params() {
        let c = RemoteCommand::parse(
            r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"system.switch_mode","params":{"mode":"pc"}}"#,
        )
        .unwrap();
        assert_eq!(c.action, RemoteAction::SwitchMode(DisplayMode::Pc));
    }
    #[test]
    fn rejects_invalid_uuid_and_unexpected_params() {
        assert!(matches!(
            RemoteCommand::parse(
                r#"{"type":"remote.command","requestId":"bad","action":"navigation.up"}"#
            ),
            Err(CoreError::InvalidMessage)
        ));
        assert!(matches!(
            RemoteCommand::parse(
                r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"navigation.up","params":{"mode":"pc"}}"#
            ),
            Err(CoreError::InvalidMessage)
        ));
    }
}
