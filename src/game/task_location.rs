use std::path::PathBuf;
use std::fs;
use crate::game::{task_config::TaskEdit};

#[derive(Debug, Clone)]
pub struct TaskLocation {
    pub target: String,
    pub line_number: usize,
    pub line: String,
    pub __origin_folder: Option<PathBuf>,
    pub __workspace_folder: Option<PathBuf>
}

impl TaskLocation {
    pub fn new() -> Self {
        Self { 
            target: String::new(), 
            line_number: 0, 
            line: String::new(), 
            __origin_folder: None, 
            __workspace_folder: None 
        }
    }

    pub fn is_folderless(&self) -> bool {
        self.__origin_folder.is_none() && self.__workspace_folder.is_none()
    }

    pub fn is_link(&self, task_mode: TaskEdit) -> bool {
        if task_mode == TaskEdit::VIEW {
            return true;
        }
        
        self.is_folderless()
    }

    
    pub fn is_import_type(&self, task_mode: TaskEdit) -> bool {
        task_mode == TaskEdit::EDIT && self.__origin_folder != None && self.__workspace_folder != None
    }
    
    pub fn is_static_type(&self, task_mode: TaskEdit) -> bool {
        if self.is_link(task_mode) {
            return false;
        }

        self.get_origin_folder() == self.get_workspace_folder()
    }
    
    pub fn set_remote_view_type(&mut self) -> &mut Self {
        self.__origin_folder = None;
        self.__workspace_folder = None;

        self
    }

    pub fn set_origin_folder(&mut self, folder: PathBuf) -> &mut Self {
        self.__origin_folder = Some(fs::canonicalize(folder).unwrap());
        self

    }
    pub fn get_origin_readme(&self) -> PathBuf {
        let origin_folder = self.get_origin_folder();
        match origin_folder {
            Some(folder) => folder.join("README.md"),
            None => PathBuf::new()
        }
    }

    pub fn set_workspace_folder(&mut self, folder: PathBuf) -> std::io::Result<&mut Self> {
        self.__workspace_folder = Some(folder.canonicalize()?);
        Ok(self)
    }
    
    pub fn set_origin_readme(&self) -> PathBuf {
        let origin_folder = self.get_origin_folder();
        if !origin_folder.is_none() {
            return origin_folder.clone().unwrap().join("README.md");
        }

        PathBuf::new()
    }


    pub fn get_origin_folder(&self) -> Option<PathBuf> {
        if !self.__origin_folder.is_none() {
            self.__origin_folder.as_ref().map(|p| fs::canonicalize(p).unwrap())
        } else {
            None
        }
    }

    pub fn get_workspace_folder(&self) -> Option<PathBuf> {
        if !self.__workspace_folder.is_none() {
            return self.__workspace_folder.as_ref().map(|p| fs::canonicalize(p).unwrap());
        }
        
        if !self.__origin_folder.is_none() {
            return self.__origin_folder.as_ref().map(|p| fs::canonicalize(p).unwrap());
        }

        None
    }
}