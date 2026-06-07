//! UIAction enum — all high-level actions that can be triggered by key bindings.
use telarex_core::command::Command as CoreCommand;

/// High-level UI action triggered by key bindings or command palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIAction {
    Core(CoreCommand),
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    ToggleExplorer,
    EnterCommandMode,
    EnterSearchMode,
    ExitMode,
    ExecuteSearch,
    SelectSearchResult,
    TriggerAutocomplete,
    ToggleMacroPalette,
    StartRecordingMacro(String),
    StopRecordingMacro,
    PlayMacro(String),
    SplitVertical,
    SplitHorizontal,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    SwitchFocus,
    EnterWindowMode,
    ClosePane,
    LeaveWorkspace,
    DisconnectNetwork,
    PromptShareWorkspace,
    Copy,
    Paste,
    Quit,
}
