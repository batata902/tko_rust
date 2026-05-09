use crate::utils::text::Text;

#[derive(Clone)]
pub struct TreeItem {
    __remote_name: String,
    __key: String,
    __sentence: Text,
    _title: String,

    ligature: Text,
    visible: bool,
    is_requirement_color: String
}

impl TreeItem {
    pub fn new() -> Self {
        Self { 
            __remote_name: "".to_string(), 
            __key: "".to_string(), 
            __sentence: Text::new(None, None),
            _title: "".to_string(), 
            ligature: Text::new(Some(" ".to_string()), None), visible: false, is_requirement_color: "".to_string() 
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

    pub fn set_sentence(&mut self, sentence: Text) -> &mut Self {
        self.__sentence = sentence;

        self
    }
}