use std::time::Duration;

use indicatif;

// separated out into its own thing for consistency

pub(crate) struct ProgressBar(indicatif::ProgressBar);

impl ProgressBar {
    pub(crate) fn new(len: u64, msg: &'static str) -> Self {
        let bar = indicatif::ProgressBar::new(len);
        bar.set_style(
            indicatif::ProgressStyle::with_template(
                "{msg} {spinner} [{elapsed}|{eta}] {bar:60.magenta/blue} ({percent}%)",
            )
            .unwrap()
            .progress_chars("═❥—")
            // .tick_strings(&[
            //     // red
            //     "\x1B[38;2;255;128;128m❤\x1B[m",
            //     "\x1B[38;2;255;128;128m❤\x1B[m",
            //     "\x1B[38;2;255;255;128m❤\x1B[m",
            //     // yellow
            //     "\x1B[38;2;255;255;128m❤\x1B[m",
            //     "\x1B[38;2;255;255;128m❤\x1B[m",
            //     "\x1B[38;2;128;255;128m❤\x1B[m",
            //     // green
            //     "\x1B[38;2;128;255;128m❤\x1B[m",
            //     "\x1B[38;2;128;255;128m❤\x1B[m",
            //     "\x1B[38;2;128;255;255m❤\x1B[m",
            //     // cyan
            //     "\x1B[38;2;128;255;255m❤\x1B[m",
            //     "\x1B[38;2;128;255;255m❤\x1B[m",
            //     "\x1B[38;2;128;255;255m❤\x1B[m",
            //     // blue
            //     "\x1B[38;2;128;128;255m❤\x1B[m",
            //     "\x1B[38;2;128;128;255m❤\x1B[m",
            //     "\x1B[38;2;128;128;255m❤\x1B[m",
            //     // magenta
            //     "\x1B[38;2;255;128;255m❤\x1B[m",
            //     "\x1B[38;2;255;128;255m❤\x1B[m",
            //     "\x1B[38;2;255;128;255m❤\x1B[m",
            // ]),
            .tick_strings(&["┏( ・_・)┛", "┏( ・_・)┛", "┗(・_・ )┓", "┗(・_・ )┓"]),
        );
        bar.enable_steady_tick(Duration::new(0, 16666666)); // 60fps
        bar.set_message(msg);
        Self(bar)
    }

    pub(crate) fn advance(&self, n: u64) {
        self.0.inc(n);
    }
}
