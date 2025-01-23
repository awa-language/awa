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

#[must_use] pub fn get_user_menu_decision() -> MenuAction {
    let decision = inquire::Select::new(
        "Select one of the following:",
        <MenuAction as strum::IntoEnumIterator>::iter().collect(),
    )
    .with_render_config(description_render_config())
    .prompt()
    .unwrap();

    decision
}

#[must_use] pub fn get_user_input() -> String {
    let input = inquire::Editor::new("Input:")
        .with_render_config(description_render_config())
        .prompt()
        .unwrap();

    input
}

fn description_render_config() -> inquire::ui::RenderConfig<'static> {
    inquire::ui::RenderConfig::default().with_canceled_prompt_indicator(
        inquire::ui::Styled::new("<skipped>").with_fg(inquire::ui::Color::DarkYellow),
    )
}
