#[derive(Debug, strum::EnumIter)]
pub enum MenuAction {
    PerformHotswap,
    ReturnToExecution, // Do nothing
}

impl std::fmt::Display for MenuAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuAction::PerformHotswap => write!(formatter, "Perform HotSwap"),
            MenuAction::ReturnToExecution => write!(formatter, "ReturnToExecution"),
        }
    }
}

/// Will prompt user with selection menu, getting action to perform
///
/// # Panics
///
/// Will panic if failed to prompt
#[must_use]
pub fn get_user_menu_decision() -> MenuAction {
    let decision = inquire::Select::new(
        "Select one of the following:",
        <MenuAction as strum::IntoEnumIterator>::iter().collect(),
    )
    .prompt()
    .unwrap();

    decision
}

/// Will prompt user with editor to get raw text input
///
/// # Panics
///
/// Will panic if failed to prompt
#[must_use]
pub fn get_user_input() -> String {
    let input = inquire::Editor::new("Input:").prompt().unwrap();

    input
}
