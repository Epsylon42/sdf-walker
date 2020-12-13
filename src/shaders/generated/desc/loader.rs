use std::path::PathBuf;
use std::time::SystemTime;

use super::SceneDesc;

pub struct SceneDescLoader {
    file: PathBuf,
    use_camera: bool,
    last_update: SystemTime,
}

impl SceneDescLoader {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        SceneDescLoader {
            file: path.into(),
            use_camera: true,
            last_update: SystemTime::now(),
        }
    }

    pub fn switch_camera(&mut self, enabled: bool) {
        self.use_camera = enabled;
    }

    pub fn load(&mut self) -> anyhow::Result<SceneDesc> {
        let source = std::fs::read(&self.file)?;
        let mut desc = SceneDesc::parse(&source)?;
        if !self.use_camera {
            desc.camera = None;
        }

        self.last_update = SystemTime::now();
        Ok(desc)
    }

    pub fn load_if_updated(&mut self) -> Option<anyhow::Result<SceneDesc>> {
        // sometimes fails, so repeat three times
        let metadata = std::fs::metadata(&self.file)
            .or_else(|_| {
                std::thread::sleep(std::time::Duration::from_millis(10));
                std::fs::metadata(&self.file)
            })
            .or_else(|_| {
                std::thread::sleep(std::time::Duration::from_millis(50));
                std::fs::metadata(&self.file)
            });

        let modified = metadata
            .expect("Could not read file metadata")
            .modified()
            .expect("Modified time not supported");
            
        if modified > self.last_update {
            let ret = self.load();
            self.last_update = modified;
            Some(ret)
        } else {
            None
        }
    }
}
