use ratzilla::ratatui::{
    Frame,
    layout::{
        Constraint::{self, Percentage, Ratio},
        Layout, Spacing,
    },
    symbols::merge::MergeStrategy,
    text::Text,
    widgets::{Block, Clear},
};

pub fn draw(frame: &mut Frame) {
    use Constraint::{Fill, Length, Min};
    let area = frame.area().centered(Percentage(70), Percentage(70));

    let vertical = Layout::vertical([Length(2), Min(0), Length(4)]);
    let [title_area, main_area, status_area] = vertical.spacing(Spacing::Overlap(1)).areas(area);
    let horizontal = Layout::horizontal([Fill(1), Fill(4)]);
    let [left_area, right_area] = horizontal.spacing(Spacing::Overlap(1)).areas(main_area);

    frame.render_widget(Clear, area);
    frame.render_widget(
        Text::raw("Joe-Ansel Puplava's Portfolio").centered(),
        title_area.centered_vertically(Ratio(1, 2)),
    );
    frame.render_widget(
        Block::bordered()
            .title("Status Bar")
            .merge_borders(MergeStrategy::Exact),
        status_area,
    );
    frame.render_widget(
        Block::bordered().merge_borders(MergeStrategy::Exact),
        left_area,
    );
    frame.render_widget(
        Block::bordered().merge_borders(MergeStrategy::Exact),
        right_area,
    );
}
