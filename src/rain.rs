//! Animated "digital rain" background.

use ratzilla::ratatui::{Frame, style::Color};

use web_time::Duration;

use tui_rain::{Rain, RainDensity, RainSpeed};

// const TAIL_COLOR: Color = Color::from_u32(0x0008dbbe);
const TAIL_COLOR: Color = Color::Cyan;

/// Draws the rain over the whole screen. The animation is computed from
/// `elapsed`, so it needs no state of its own.
pub fn view(frame: &mut Frame, elapsed: Duration) {
    let rain = Rain::new_rain(elapsed)
        .with_rain_density(RainDensity::Relative { sparseness: 40 })
        .with_rain_speed(RainSpeed::Absolute { speed: 10.0 })
        .with_rain_speed_variance(0.6)
        .with_color(TAIL_COLOR)
        .with_noise_interval(Duration::from_secs(10));

    frame.render_widget(rain, frame.area());
}
