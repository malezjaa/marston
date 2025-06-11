use crate::MPath;
use ariadne::{Report, Source};
use once_cell::sync::Lazy;
use std::{
    borrow::Cow,
    ops::Range,
    sync::{Arc, Mutex},
};

pub type MReport = Report<'static, (Arc<MPath>, Range<usize>)>;

#[derive(Debug)]
pub struct ReportsBag {
    reports: Vec<MReport>,
    file: Arc<MPath>,
    source_content: Arc<String>,
}

impl ReportsBag {
    fn new(file_name: Arc<MPath>, source_content: Arc<String>) -> Self {
        Self { reports: Vec::new(), file: file_name, source_content }
    }

    fn global_mut() -> std::sync::MutexGuard<'static, Self> {
        REPORTS_BAG.lock().expect("Failed to lock REPORTS_BAG")
    }

    pub fn init(file_name: Arc<MPath>, source_content: Arc<String>) {
        let mut bag = Self::global_mut();
        *bag = Self::new(file_name, source_content);
    }

    pub fn add(report: MReport) {
        Self::global_mut().reports.push(report);
    }

    pub fn print() {
        let bag = Self::global_mut();
        for report in &bag.reports {
            let _ = report.print((bag.file.clone(), Source::from(bag.source_content.as_str())));
        }
    }

    pub fn has_errors() -> bool {
        !Self::global_mut().reports.is_empty()
    }

    pub fn clear_errors() {
        Self::global_mut().reports.clear();
    }

    pub fn file() -> Arc<MPath> {
        Self::global_mut().file.clone()
    }
}

pub static REPORTS_BAG: Lazy<Mutex<ReportsBag>> = Lazy::new(|| {
    let dummy_path = Arc::new(MPath::new());
    let dummy_source = Arc::new(String::new());
    Mutex::new(ReportsBag::new(dummy_path, dummy_source))
});

#[macro_export]
macro_rules! error_report {
    (
        span: $span:expr,
        message: $message:expr
        $(, labels: {
            $($label_span:expr => {
                message: $label_msg:expr => $label_color:expr
            }),* $(,)?
        })?
        $(, notes: [$($note:expr),* $(,)?])?
        $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut report = Report::build(ReportKind::Error, (ReportsBag::file(), $span))
            .with_message($message);

        $(
            $(
                let label = Label::new((ReportsBag::file(), $label_span))
                    .with_message($label_msg).with_color($label_color);

                report = report.with_label(label);
            )*
        )?

        $(
            $(
                report = report.with_note($note);
            )*
        )?

        report.finish()
    }};
}
