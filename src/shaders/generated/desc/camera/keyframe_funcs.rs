use super::{Keyframe, Rotation};
use crate::shaders::generated::desc::Statement;

pub fn parse_keyframe(stmt: Statement) -> Keyframe {
    assert_eq!(stmt.name, "keyframe");
    assert!(
        stmt.args.len() > 0,
        "keyframe must have at least one argument"
    );

    let t = stmt.args[0].parse().unwrap();

    let mut pos = None;
    let mut rot = None;

    for stmt in &stmt.body {
        match parse_keyframe_arg(stmt) {
            KeyframeArg::Position(x) => {
                if pos.is_some() {
                    panic!("duplicate position inside keyframe");
                } else {
                    pos = Some(x);
                }
            }

            KeyframeArg::Rotation(x) => {
                if rot.is_some() {
                    panic!("duplicate rotation inside keyframe");
                } else {
                    rot = Some(x);
                }
            }
        }
    }

    Keyframe {
        t,
        pos: pos.into(),
        rot: rot.into(),
    }
}

enum KeyframeArg {
    Position(glm::Vec3),
    Rotation(Rotation),
}

fn parse_keyframe_arg(stmt: &Statement) -> KeyframeArg {
    assert!(stmt.body.is_empty());

    match stmt.name.as_str() {
        "pos" => {
            assert_eq!(stmt.args.len(), 3);

            let pos = glm::Vec3::new(
                stmt.args[0].parse().unwrap(),
                stmt.args[1].parse().unwrap(),
                stmt.args[2].parse().unwrap(),
            );

            KeyframeArg::Position(pos)
        }

        "look_at" => {
            assert_eq!(stmt.args.len(), 3);

            let pos = glm::Vec3::new(
                stmt.args[0].parse().unwrap(),
                stmt.args[1].parse().unwrap(),
                stmt.args[2].parse().unwrap(),
            );

            KeyframeArg::Rotation(Rotation::LookAt(pos))
        }

        "euler" => {
            assert!(stmt.args.len() >= 3);

            let unit: fn(f32) -> f32 = if let Some(unit) = stmt.args.get(3) {
                match unit.as_str() {
                    "degrees" => |x: f32| x.to_radians(),
                    "radians" => |x: f32| x,
                    unit => panic!("Unknown angle unit: '{}'", unit),
                }
            } else {
                |x: f32| x.to_radians()
            };

            let pitch = stmt.args[0].parse().unwrap();
            let yaw = stmt.args[1].parse().unwrap();
            let roll = stmt.args[2].parse().unwrap();

            let mut quat = glm::Quat::new(1.0, 0.0, 0.0, 0.0);
            quat = glm::quat_rotate(&quat, unit(pitch), &glm::Vec3::x());
            quat = glm::quat_rotate(&quat, unit(yaw), &glm::Vec3::y());
            quat = glm::quat_rotate(&quat, unit(roll), &glm::Vec3::z());

            KeyframeArg::Rotation(Rotation::Absolute(quat))
        }

        "quat" | "quaternion" => {
            assert_eq!(stmt.args.len(), 4);

            let quat = glm::Quat::new(
                stmt.args[0].parse().unwrap(),
                stmt.args[1].parse().unwrap(),
                stmt.args[2].parse().unwrap(),
                stmt.args[3].parse().unwrap(),
            );

            KeyframeArg::Rotation(Rotation::Absolute(quat))
        }

        _ => {
            panic!("'{}': unknown keyframe argument", stmt.name);
        }
    }
}
