//! Command enumeration — a typed set of every action the editor can perform.
//!
//! [`Command`] covers file operations, git integration, workspace sharing,
//! and application lifecycle. It provides human-readable names and descriptions
//! for use in command palettes and keybinding displays.

/// A typed action the editor can perform — file ops, git, network, and lifecycle.
///
/// Each variant has a human-readable [`name`](Command::name) and
/// [`description`](Command::description) for UI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    OpenFile,
    Save,
    SaveAs,
    Quit,
    ToggleExplorer,
    OpenConfig,
    ShareWorkspace,
    LeaveWorkspace,
    DisconnectNetwork,
    ResetData,
    GitStatus,
    GitStageAll,
    GitCommit,
    GitPush,
    GitPull,
    GitLog,
}

impl Command {
    /// Return every available command variant.
    pub fn all() -> Vec<Command> {
        vec![
            Command::OpenFile,
            Command::Save,
            Command::ToggleExplorer,
            Command::OpenConfig,
            Command::ShareWorkspace,
            Command::LeaveWorkspace,
            Command::DisconnectNetwork,
            Command::ResetData,
            Command::GitStatus,
            Command::GitStageAll,
            Command::GitCommit,
            Command::GitPush,
            Command::GitPull,
            Command::GitLog,
            Command::Quit,
        ]
    }

    /// Return a short, human-readable name for this command.
    pub fn name(&self) -> &'static str {
        match self {
            Command::OpenFile => "Open File",
            Command::Save => "Save",
            Command::SaveAs => "Save As",
            Command::Quit => "Quit",
            Command::ToggleExplorer => "Toggle File Explorer",
            Command::OpenConfig => "Open Configuration",
            Command::ShareWorkspace => "Share Workspace (Lodge)",
            Command::LeaveWorkspace => "Leave Lodge",
            Command::DisconnectNetwork => "Disconnect from Network",
            Command::ResetData => "Reset Application Data",
            Command::GitStatus => "Git Status",
            Command::GitStageAll => "Git Stage All",
            Command::GitCommit => "Git Commit",
            Command::GitPush => "Git Push",
            Command::GitPull => "Git Pull",
            Command::GitLog => "Git Log",
        }
    }

    /// Return a longer description explaining what this command does.
    pub fn description(&self) -> &'static str {
        match self {
            Command::OpenFile => "Open a file from the filesystem",
            Command::Save => "Save the current file",
            Command::SaveAs => "Save the current file with a new name",
            Command::Quit => "Exit TelaRex",
            Command::ToggleExplorer => "Show or hide the file explorer panel",
            Command::OpenConfig => "Open TelaRex settings",
            Command::ShareWorkspace => "Broadcast this workspace to the network as a Lodge",
            Command::LeaveWorkspace => "Stop sharing or exit current lodge",
            Command::DisconnectNetwork => "Shutdown all peer-to-peer connectivity",
            Command::ResetData => "Wipe all local configuration and database records",
            Command::GitStatus => "Show working tree status in the log",
            Command::GitStageAll => "Stage all changes for commit",
            Command::GitCommit => "Commit staged changes with a message",
            Command::GitPush => "Push commits to remote origin",
            Command::GitPull => "Fetch and merge from remote origin",
            Command::GitLog => "Show recent commit history",
        }
    }
}
