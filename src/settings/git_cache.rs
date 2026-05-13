use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime},
};

use sha1::{Digest, Sha1};
use crate::settings::file_lock::FileLock;

pub struct GitCache {
    cache_dir: PathBuf,
    update_mode: UpdateMode,
    updated: HashMap<String, bool>,
    max_age: Duration,
}

#[derive(PartialEq)]
pub enum UpdateMode {
    ALWAYS,
    NEVER,
    IfOlder,
}

impl GitCache {
    pub fn new(
        cache_dir: impl Into<PathBuf>,
        max_age: Option<Duration>,
        update_mode: Option<UpdateMode>,
    ) -> std::io::Result<Self> {
        let cache_dir = cache_dir.into();
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            update_mode: update_mode.unwrap_or(UpdateMode::IfOlder),
            updated: HashMap::new(),
            max_age: max_age.unwrap_or(Duration::from_secs(3600)),
        })
    }

    pub fn clear_cache(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            println!("Clearing git cache at {}...", self.cache_dir.display());
            fs::remove_dir_all(&self.cache_dir)?;
        }
        fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    pub fn __repo_dir(&self, url: &str) -> PathBuf {
        let mut hasher = Sha1::new();
        hasher.update(url.as_bytes());
        let digest = hex::encode(hasher.finalize());

        self.cache_dir.join(digest)
    }

    pub fn __lock_path(&self, repo_path: &Path) -> PathBuf {
        repo_path.with_extension("lock")
    }

    fn _git(&self, args: &[&str], cwd: Option<&Path>) -> Result<(), String> {
        let mut cmd = Command::new("git");

        cmd.args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd.output().map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn clone_repo(&self, url: &str, path: &Path) -> Result<(), String> {
        self.git(
            &[
                "clone",
                "--depth",
                "1",
                "--filter=blob:none",
                "--no-single-branch",
                url,
                path.to_str().unwrap(),
            ],
            None,
        )
    }

    pub fn update(&mut self, repo: &Path) -> Result<(), String> {
        self.git(&["fetch", "--prune", "origin"], Some(repo))?;
        self.git(&["reset", "--hard", "origin/HEAD"], Some(repo))?;
        self.git(&["clean", "-fd"], Some(repo))?;

        self.updated
            .insert(repo.to_string_lossy().to_string(), true);

        Ok(())
    }

    pub fn is_expired(&self, repo: &Path) -> bool {
        let fetch_head = repo.join(".git").join("FETCH_HEAD");

        if !fetch_head.exists() {
            return true;
        }

        let metadata = match fs::metadata(&fetch_head) {
            Ok(m) => m,
            Err(_) => return true,
        };

        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(_) => return true,
        };

        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default();

        age > self.max_age
    }

    pub fn __acquire_lock(&self, lock_path: &Path) -> std::io::Result<FileLock> {
        Ok(FileLock::new(lock_path).unwrap())
    }

    pub fn get(&mut self, url: &str) -> Result<PathBuf, String> {
        let repo = self.repo_dir(url);
        let lock_path = self.lock_path(&repo);

        let _lock = self.acquire_lock(&lock_path).map_err(|e| e.to_string())?;

        // clone
        if !repo.exists() {
            println!("Cloning {} into cache...", url);
            self.clone_repo(url, &repo)?;
            return Ok(repo);
        }

        // should update?
        let updated_flag = self
            .updated
            .get(&repo.to_string_lossy().to_string())
            .copied()
            .unwrap_or(false);

        let should_update = self.is_expired(&repo)
            || (self.update_mode == UpdateMode::ALWAYS && !updated_flag);

        if should_update {
            println!("Updating cache for {}...", url);

            if let Err(_) = self.update(&repo) {
                println!("Failed to update. Re-cloning...");

                let _ = fs::remove_dir_all(&repo);
                self.clone_repo(url, &repo)?;
            }
        }

        Ok(repo)
    }

    pub fn __clone(&self, url: &str, path: &Path) -> bool {
        match self._git(
            &["clone", "--depth", "1", "--filter=blob:none", "--no-single-branch"], 
            url,
                path.to_str()
        ) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn get_repo_dir(&self, url: &str, verbose: bool) -> Option<&Path> {
        let repo: PathBuf = self.__repo_dir(url);
        let lock_path: PathBuf = self.__lock_path(repo.as_path());

        {
            let _lock = self.__acquire_lock(&lock_path);
            if !repo.exists() {
                if verbose {
                    eprintln!("Cloning {} into cache...", url);
                }
                let ok = self.
            }
        }

    }
}