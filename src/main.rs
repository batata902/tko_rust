use std::borrow::Cow;
use std::path::PathBuf;

mod settings;
mod utils;
mod play;
mod game;
mod feno;

use utils::decoder::decoder;
use utils::get_md_link::get_md_link;
use utils::symbols;
use utils::text::{AddValue, AnsiColor, Text};

use game::quest::Quest;
use game::quest_parser::QuestParser;
use game::task::Task;
use game::task_parser::TaskParser;
use game::tree_item::TreeItem;

use settings::git_cache::{GitCache, UpdateMode};
use settings::rep_source::RepSource;
use settings::settings::Settings;

use play::tasktree::TreeFilter;

use down::sandbox_drafts::SandboxDrafts;

mod down {
    pub mod sandbox_drafts;
}

fn main() {
    
}