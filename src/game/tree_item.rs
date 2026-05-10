use crate::utils::rtext::RText;

#[derive(Clone)]
pub struct TreeUi {
    pub ligature: RText,
    pub visible: bool,
    pub is_requirement_color: String
}


impl TreeUi {
    pub fn new() -> Self {
        Self { 
            ligature: RText::new(), 
            visible: false, 
            is_requirement_color: String::new() 
        }
    }
}


#[derive(Clone)]
pub struct TreeItem {
    __remote_name: String,
    __key: String,
    _title: String,
}

impl TreeItem {
    pub fn new() -> Self {
        Self { 
            __remote_name: "".to_string(), 
            __key: "".to_string(), 
            _title: "".to_string()
        }
    }

    pub fn get_remote_name(&self) -> &str {
        &self.__remote_name
    }

    pub fn get_full_key(&self) -> String {
        format!("{}@{}", &self.__remote_name, &self.__key)
    }
    
    pub fn get_key(&self) -> &str {
        &self.__key
    }

    pub fn get_title(&self) -> &str {
        &self._title
    }


    pub fn set_remote_name(&mut self, remote_name: &str) -> &mut Self {
        self.__remote_name = remote_name.to_string();
        
        self
    }

    pub fn set_key(&mut self, mut key: String) -> &mut Self {
        if key.starts_with("@") {
            key.remove(0);
        }
        self.__key = key;

        self
    }

    pub fn set_title(&mut self, title: String) -> &mut Self {
        self._title = title;
        self
    }

    // pub fn set_sentence(&mut self, sentence: Text) -> &mut Self {
    //     self.__sentence = sentence;

    //     self
    // }
}

pub trait HasTreeIdentity {
    fn identity(&self) -> &TreeItem;
}

impl HasTreeIdentity for TreeItem {
    fn identity(&self) -> &TreeItem {
        self
    }
}