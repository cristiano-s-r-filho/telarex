pub mod editor;
pub mod explorer;
pub mod bars;
pub mod palettes;
pub mod modals;
pub mod layout;
pub mod tab_controller;

pub use editor::Editor;
pub use explorer::FileTree;
pub use bars::status_bar::StatusBar;
#[allow(unused_imports)]
pub use bars::tab_bar::TabBar;
pub use palettes::command::CommandPalette;
pub use palettes::search::{SearchPalette, SearchResult};
pub use layout::NodeKind;
pub use modals::ErrorModal;
pub use tab_controller::TabController;
