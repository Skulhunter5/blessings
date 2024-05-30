use crossterm::cursor::SetCursorStyle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    DefaultUserShape,
    BlinkingBlock,
    SteadyBlock,
    BlinkingUnderScore,
    SteadyUnderScore,
    BlinkingBar,
    SteadyBar,
}

impl CursorStyle {
    pub fn to_crossterm_command(&self) -> SetCursorStyle {
        match self {
            CursorStyle::DefaultUserShape => SetCursorStyle::DefaultUserShape,
            CursorStyle::BlinkingBlock => SetCursorStyle::BlinkingBlock,
            CursorStyle::SteadyBlock => SetCursorStyle::SteadyBlock,
            CursorStyle::BlinkingUnderScore => SetCursorStyle::BlinkingUnderScore,
            CursorStyle::SteadyUnderScore => SetCursorStyle::SteadyUnderScore,
            CursorStyle::BlinkingBar => SetCursorStyle::BlinkingBar,
            CursorStyle::SteadyBar => SetCursorStyle::SteadyBar,
        }
    }
}

impl Into<SetCursorStyle> for CursorStyle {
    fn into(self) -> SetCursorStyle {
        self.to_crossterm_command()
    }
}
