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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaAction {
    PlayPause,
    SeekBackward,
    SeekForward,
    FullscreenToggle,
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
    Media(MediaAction),
    SwitchMode(DisplayMode),
    Scroll { dx: i32, dy: i32 },
    PointerMove { dx: i32, dy: i32 },
    PointerClick,
    InputText(String),
    DeleteBackward,
    SubmitText,
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
    mode: Option<DisplayMode>,
    text: Option<String>,
    dx: Option<i32>,
    dy: Option<i32>,
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
            "media.play_pause" => RemoteAction::Media(MediaAction::PlayPause),
            "media.seek_backward" => RemoteAction::Media(MediaAction::SeekBackward),
            "media.seek_forward" => RemoteAction::Media(MediaAction::SeekForward),
            "media.fullscreen_toggle" => RemoteAction::Media(MediaAction::FullscreenToggle),
            "system.switch_mode" => {
                let params = dto.params.as_ref().ok_or(CoreError::InvalidMessage)?;
                RemoteAction::SwitchMode(params.mode.ok_or(CoreError::InvalidMessage)?)
            }
            "pointer.scroll" => {
                let params = dto.params.as_ref().ok_or(CoreError::InvalidMessage)?;
                let dx = params.dx.ok_or(CoreError::InvalidMessage)?;
                let dy = params.dy.ok_or(CoreError::InvalidMessage)?;
                if (dx == 0 && dy == 0) || dx.unsigned_abs() > 1200 || dy.unsigned_abs() > 1200 {
                    return Err(CoreError::InvalidMessage);
                }
                RemoteAction::Scroll { dx, dy }
            }
            "pointer.move" => {
                let params = dto.params.as_ref().ok_or(CoreError::InvalidMessage)?;
                let dx = params.dx.ok_or(CoreError::InvalidMessage)?;
                let dy = params.dy.ok_or(CoreError::InvalidMessage)?;
                if (dx == 0 && dy == 0) || dx.unsigned_abs() > 1200 || dy.unsigned_abs() > 1200 {
                    return Err(CoreError::InvalidMessage);
                }
                RemoteAction::PointerMove { dx, dy }
            }
            "pointer.click" => RemoteAction::PointerClick,
            "input.text" => {
                let params = dto.params.as_ref().ok_or(CoreError::InvalidMessage)?;
                let text = params.text.as_ref().ok_or(CoreError::InvalidMessage)?;
                if text.is_empty()
                    || text.chars().count() > 200
                    || text.chars().any(char::is_control)
                {
                    return Err(CoreError::InvalidMessage);
                }
                RemoteAction::InputText(text.clone())
            }
            "input.delete_backward" => RemoteAction::DeleteBackward,
            "input.submit" => RemoteAction::SubmitText,
            _ => return Err(CoreError::InvalidCommand),
        };
        let params_are_valid = match &action {
            RemoteAction::SwitchMode(_) => dto.params.as_ref().is_some_and(|params| {
                params.text.is_none() && params.dx.is_none() && params.dy.is_none()
            }),
            RemoteAction::Scroll { .. } | RemoteAction::PointerMove { .. } => {
                dto.params.as_ref().is_some_and(|params| {
                    params.mode.is_none()
                        && params.text.is_none()
                        && params.dx.is_some()
                        && params.dy.is_some()
                })
            }
            RemoteAction::InputText(_) => dto.params.as_ref().is_some_and(|params| {
                params.mode.is_none() && params.dx.is_none() && params.dy.is_none()
            }),
            _ => dto.params.is_none(),
        };
        if !params_are_valid {
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

    #[test]
    fn parses_unicode_text_and_rejects_invalid_text() {
        let command = RemoteCommand::parse(
            r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"input.text","params":{"text":"すいか🍉"}}"#,
        )
        .unwrap();
        assert_eq!(command.action, RemoteAction::InputText("すいか🍉".into()));
        assert!(RemoteCommand::parse(
            r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"input.text","params":{"text":""}}"#,
        )
        .is_err());
    }

    #[test]
    fn parses_bounded_scroll_and_rejects_invalid_deltas() {
        let command = RemoteCommand::parse(
            r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"pointer.scroll","params":{"dx":-240,"dy":360}}"#,
        )
        .unwrap();
        assert_eq!(command.action, RemoteAction::Scroll { dx: -240, dy: 360 });
        for params in [
            r#"{"dx":0,"dy":0}"#,
            r#"{"dx":0,"dy":1201}"#,
            r#"{"dx":-1201,"dy":0}"#,
        ] {
            let message = format!(
                r#"{{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"pointer.scroll","params":{params}}}"#
            );
            assert!(RemoteCommand::parse(&message).is_err());
        }
    }

    #[test]
    fn parses_pointer_move_and_click() {
        let movement = RemoteCommand::parse(
            r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"pointer.move","params":{"dx":20,"dy":-15}}"#,
        )
        .unwrap();
        assert_eq!(
            movement.action,
            RemoteAction::PointerMove { dx: 20, dy: -15 }
        );
        let click = RemoteCommand::parse(
            r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"pointer.click"}"#,
        )
        .unwrap();
        assert_eq!(click.action, RemoteAction::PointerClick);
    }

    #[test]
    fn parses_media_actions_without_params() {
        for (action, expected) in [
            ("media.play_pause", MediaAction::PlayPause),
            ("media.seek_backward", MediaAction::SeekBackward),
            ("media.seek_forward", MediaAction::SeekForward),
            ("media.fullscreen_toggle", MediaAction::FullscreenToggle),
        ] {
            let message = format!(
                r#"{{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"{action}"}}"#
            );
            assert_eq!(
                RemoteCommand::parse(&message).unwrap().action,
                RemoteAction::Media(expected)
            );
        }
        assert!(RemoteCommand::parse(
            r#"{"type":"remote.command","requestId":"550e8400-e29b-41d4-a716-446655440000","action":"media.play_pause","params":{"dx":1}}"#
        ).is_err());
    }
}
