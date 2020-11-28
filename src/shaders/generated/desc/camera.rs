use super::Statement;

use std::collections::HashMap;

mod keyframe;
mod marker;

use keyframe::{Keyframe, KeyframeError};
use marker::MarkerError;

#[derive(Debug, Clone, Copy)]
pub enum Param<T: Clone + Copy> {
    Override(T),
    Reuse,
}

impl<T: Default + Clone + Copy> Param<T> {
    fn get(self) -> T {
        match self {
            Param::Override(x) => x,
            Param::Reuse => T::default(),
        }
    }
}

impl<T: Clone + Copy> From<Option<T>> for Param<T> {
    fn from(opt: Option<T>) -> Param<T> {
        opt.map(Param::Override).unwrap_or(Param::Reuse)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Absolute(glm::Quat),
    LookAt(glm::Vec3),
}

impl Rotation {
    fn to_quat(self, pos: glm::Vec3) -> glm::Quat {
        match self {
            Rotation::Absolute(x) => x,
            Rotation::LookAt(x) => {
                let dir = (x - pos).normalize();
                glm::quat_look_at_lh(&dir, &glm::Vec3::y())
            }
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::Absolute(glm::Quat::default())
    }
}


#[derive(Debug, Clone)]
pub struct CameraDesc {
    timeline: Vec<Keyframe>,
    markers: HashMap<String, glm::Vec3>,
}

impl Default for CameraDesc {
    fn default() -> Self {
        CameraDesc {
            timeline: Vec::new(),
            markers: HashMap::new(),
        }
    }
}

impl CameraDesc {
    pub fn new(stmt: Statement) -> Result<CameraDesc, CameraDescError> {
        assert_eq!(stmt.name, "camera");
        assert!(stmt.args.is_empty(), "Camera does not take arguments");

        let mut timeline = Vec::new();
        let mut markers = HashMap::new();

        for stmt in stmt.body {
            match stmt.name.as_str() {
                "keyframe" => timeline.push(Keyframe::new(stmt)?),
                "marker" => {
                    let (k, v) = marker::parse_marker(stmt)?;
                    markers.insert(k, v);
                }

                x => return Err(CameraDescError::UnknownStatement(x.into()))
            }
        }

        Ok(
            CameraDesc {
                timeline,
                markers,
            }
        )
    }

    pub fn duration(&self) -> f32 {
        if self.timeline.is_empty() {
            0.0
        } else {
            self.timeline[self.timeline.len() - 1].t
        }
    }

    pub fn get_pos_at_frame(&self, frame: usize) -> glm::Vec3 {
        match self.timeline.len() {
            0 => Default::default(),
            1 => self.timeline[0].pos_with_marker(&self.markers).get(),
            _ => match self.timeline[frame].pos_with_marker(&self.markers) {
                Param::Override(x) => x,
                Param::Reuse if frame == 0 => Default::default(),
                Param::Reuse => self.get_pos_at_frame(frame - 1),
            },
        }
    }

    pub fn get_rot_at_frame(&self, frame: usize) -> Rotation {
        match self.timeline.len() {
            0 => Default::default(),
            1 => self.timeline[0].rot_with_marker(&self.markers).get(),
            _ => match self.timeline[frame].rot_with_marker(&self.markers) {
                Param::Override(x) => x,
                Param::Reuse if frame == 0 => Default::default(),
                Param::Reuse => self.get_rot_at_frame(frame - 1),
            },
        }
    }

    pub fn get_transform_at(&self, t: f32) -> (glm::Vec3, glm::Quat) {
        let frame_idx = self
            .timeline
            .binary_search_by(|kf| kf.t.partial_cmp(&t).unwrap())
            .unwrap_or_else(|closest_idx| closest_idx);

        if frame_idx == 0 {
            let kf = self.timeline.get(0).cloned().unwrap_or_default();

            let pos = kf.pos.get();
            (pos, kf.rot.get().to_quat(pos))
        } else if frame_idx >= self.timeline.len() {
            let pos = self.get_pos_at_frame(frame_idx - 1);
            (pos, self.get_rot_at_frame(frame_idx - 1).to_quat(pos))
        } else {
            let kf1 = &self.timeline[frame_idx - 1];
            let kf2 = &self.timeline[frame_idx];
            let a = (t - kf1.t) / (kf2.t - kf1.t);

            let pos1 = self.get_pos_at_frame(frame_idx - 1);
            let pos = match self.timeline[frame_idx].pos {
                Param::Reuse => pos1,
                Param::Override(pos2) => glm::mix(&pos1, &pos2, a),
            };

            let rot1 = self.get_rot_at_frame(frame_idx - 1).to_quat(pos);
            let rot = match self.timeline[frame_idx].rot {
                Param::Reuse => rot1,
                Param::Override(rot2) => glm::quat_fast_mix(&rot1, &rot2.to_quat(pos), a),
            };

            (pos, rot)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CameraDescError {
    #[error("Unknown statement: '{}'", .0)]
    UnknownStatement(String),
    #[error("{}", .0)]
    Keyframe(#[from] KeyframeError),
    #[error("{}", .0)]
    Marker(#[from] MarkerError),
}
