use ratzilla::ratatui::{
    Frame,
    layout::{
        Constraint::{self, Percentage, Ratio},
        Layout,
    },
    text::Text,
    widgets::Block,
};

pub fn draw(frame: &mut Frame) {
    use Constraint::{Fill, Length, Min};

    let vertical = Layout::vertical([Length(2), Min(0), Length(2)]);
    let [title_area, main_area, status_area] =
        vertical.areas(frame.area().centered(Percentage(70), Percentage(70)));
    let horizontal = Layout::horizontal([Fill(1), Fill(4)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    frame.render_widget(
        Text::raw("Joe-Ansel Puplava's Portfolio").centered(),
        title_area,
    );
    frame.render_widget(Block::bordered().title("Status Bar"), status_area);
    frame.render_widget(Block::bordered().title("Left"), left_area);
    frame.render_widget(Block::bordered().title("Right"), right_area);
}
