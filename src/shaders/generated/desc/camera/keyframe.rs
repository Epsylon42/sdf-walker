use super::{Rotation, Param};
use crate::shaders::generated::desc::Statement;

use std::num::ParseFloatError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub t: f32,
    pub marker: Option<String>,
    pub pos: Param<glm::Vec3>,
    pub rot: Param<Rotation>,
}

impl Keyframe {
    pub fn new(stmt: Statement, prev_t: f32) -> Result<Keyframe, KeyframeError> {
        parse_keyframe(stmt, prev_t)
    }

    pub fn pos_with_marker(&self, markers: &HashMap<String, glm::Vec3>) -> Param<glm::Vec3> {
        if let Param::Override(pos) = self.pos {
            let pos = self.marker
                .as_ref()
                .and_then(|marker| markers.get(marker))
                .map(|marker| marker + pos)
                .unwrap_or(pos);

            Param::Override(pos)
        } else {
            self.pos
        }
    }

    pub fn rot_with_marker(&self, markers: &HashMap<String, glm::Vec3>) -> Param<Rotation> {
        if let Param::Override(Rotation::LookAt(look_at)) = self.rot {
            let look_at = self.marker
                .as_ref()
                .and_then(|marker| markers.get(marker))
                .map(|marker| marker + look_at)
                .unwrap_or(look_at);

            Param::Override(Rotation::LookAt(look_at))
        } else {
            self.rot
        }
    }
}

impl Default for Keyframe {
    fn default() -> Self {
        Keyframe {
            t: 0.0,
            marker: None,
            pos: Param::Reuse,
            rot: Param::Reuse,
        }
    }
}

pub fn parse_keyframe(stmt: Statement, prev_t: f32) -> Result<Keyframe, KeyframeError> {
    assert_eq!(stmt.name, "keyframe");
    if stmt.args.is_empty() {
        return Err(KeyframeError::NoArgs);
    }

    let mut t = stmt.args[0].parse()?;
    if stmt.args.len() > 1 && stmt.args[1] == "+" {
        t += prev_t;
    }

    let mut marker = None;
    let mut pos = None;
    let mut rot = None;

    for mut stmt in stmt.body {
        if stmt.args.len() >= 1 && stmt.args[0].starts_with('$') {
            let marker_name = stmt.args.remove(0);
            marker = Some(marker_name[1..].to_owned());
        }

        match parse_keyframe_arg(stmt)? {
            KeyframeArg::Position(x) => {
                if pos.is_some() {
                    return Err(KeyframeError::Duplicate("position"))
                } else {
                    pos = Some(x);
                }
            }

            KeyframeArg::Rotation(x) => {
                if rot.is_some() {
                    return Err(KeyframeError::Duplicate("rotation"))
                } else {
                    rot = Some(x);
                }
            }
        }
    }

    Ok(
        Keyframe {
            t,
            marker,
            pos: pos.into(),
            rot: rot.into(),
        }
    )
}

enum KeyframeArg {
    Position(glm::Vec3),
    Rotation(Rotation),
}

fn parse_keyframe_arg(stmt: Statement) -> Result<KeyframeArg, KeyframeError> {
    assert!(stmt.body.is_empty());

    let arg = match stmt.name.as_str() {
        "pos" => {
            assert_eq!(stmt.args.len(), 3);

            let pos = glm::Vec3::new(
                stmt.args[0].parse()?,
                stmt.args[1].parse()?,
                stmt.args[2].parse()?,
            );

            KeyframeArg::Position(pos)
        }

        "look_at" => {
            assert_eq!(stmt.args.len(), 3);

            let pos = glm::Vec3::new(
                stmt.args[0].parse()?,
                stmt.args[1].parse()?,
                stmt.args[2].parse()?,
            );

            KeyframeArg::Rotation(Rotation::LookAt(pos))
        }

        "euler" => {
            assert!(stmt.args.len() >= 3);

            let unit: fn(f32) -> f32 = if let Some(unit) = stmt.args.get(3) {
                match unit.as_str() {
                    "degrees" => |x: f32| x.to_radians(),
                    "radians" => |x: f32| x,
                    unit => return Err(KeyframeError::UnknownUnit(unit.into())),
                }
            } else {
                |x: f32| x.to_radians()
            };

            let pitch = stmt.args[0].parse()?;
            let yaw = stmt.args[1].parse()?;
            let roll = stmt.args[2].parse()?;

            let mut quat = glm::Quat::new(1.0, 0.0, 0.0, 0.0);
            quat = glm::quat_rotate(&quat, unit(pitch), &glm::Vec3::x());
            quat = glm::quat_rotate(&quat, unit(yaw), &glm::Vec3::y());
            quat = glm::quat_rotate(&quat, unit(roll), &glm::Vec3::z());

            KeyframeArg::Rotation(Rotation::Absolute(quat))
        }

        "quat" | "quaternion" => {
            assert_eq!(stmt.args.len(), 4);

            let quat = glm::Quat::new(
                stmt.args[0].parse()?,
                stmt.args[1].parse()?,
                stmt.args[2].parse()?,
                stmt.args[3].parse()?,
            );

            KeyframeArg::Rotation(Rotation::Absolute(quat))
        }

        _ => return Err(KeyframeError::UnknownArgument(stmt.name)),
    };

    Ok(arg)
}

#[derive(Debug, thiserror::Error)]
pub enum KeyframeError {
    #[error("Keyframe must have at least one argument")]
    NoArgs,
    #[error("Duplicate {} inside keyframe", .0)]
    Duplicate(&'static str),
    #[error("Unknown unit: {}", .0)]
    UnknownUnit(String),
    #[error("Unknown keyframe argument: {}", .0)]
    UnknownArgument(String),
    #[error("Failed to parse a number: {}", .0)]
    NumberParseError(#[from] ParseFloatError)
}
