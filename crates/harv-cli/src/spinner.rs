use core::future::Future;
use indicatif::{ProgressBar, ProgressStyle};

pub(crate) fn new_spinner(message: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

pub(crate) async fn with_spinner<T, E>(
    message: &'static str,
    fut: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let pb = new_spinner(message);
    let result = fut.await;
    pb.finish_and_clear();
    result
}
