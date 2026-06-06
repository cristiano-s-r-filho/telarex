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
