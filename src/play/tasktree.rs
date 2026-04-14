use crate::{game::tree_item::TreeItem};


pub struct TreeBuilder {

}

pub struct TaskTree {
    items: Vec<TreeItem>
}

pub struct TreeFilter {
    inbox_mode: bool,
    search_text: String
}

impl TreeFilter {
    pub fn new(inbox_mode: bool, search_text: String) -> Self {
        Self {
            inbox_mode,
            search_text
        }
    }

    pub fn hide_element(&self) -> bool {
        self.inbox_mode && self.search_text == ""
    }
}



// Update
// Recebe self, force_view_all (bool), ligatures (bool)
// Dependencias: TreeFilter

// TreeFilter - Classe:
//     expanded
 
