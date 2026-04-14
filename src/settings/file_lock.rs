use std::fs::File;
use std::path::{Path, PathBuf};
use fs2::FileExt;

pub struct FileLock {
    file: File,
    _path: PathBuf
}

impl FileLock {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        let path_buf = path.as_ref().to_path_buf();

        match File::create(&path_buf) {
            Ok(file) => {
                match file.lock_exclusive() {
                    Ok(_) => Ok(Self { file, _path: path_buf }),
                    Err(_) => Err(())
                }
            },
            Err(_) => Err(())
        }
        
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}