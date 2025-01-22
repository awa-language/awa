pub fn get_user_input() -> String {
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
