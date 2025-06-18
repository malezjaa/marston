use crate::{MPath, Span};
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
    source_content: Arc<str>,
    pub has_errors: bool,
}

impl ReportsBag {
    fn new(file_name: Arc<MPath>, source_content: Arc<str>) -> Self {
        Self { reports: Vec::new(), file: file_name, source_content, has_errors: false }
    }

    fn global_mut() -> std::sync::MutexGuard<'static, Self> {
        REPORTS_BAG.lock().expect("Failed to lock REPORTS_BAG")
    }

    pub fn init(file_name: Arc<MPath>, source_content: Arc<str>) {
        let mut bag = Self::global_mut();
        *bag = Self::new(file_name, source_content);
    }

    pub fn add(report: MReport) {
        Self::global_mut().reports.push(report);
    }

    pub fn print() {
        let bag = Self::global_mut();
        for report in &bag.reports {
            let _ =
                report.eprint((Arc::clone(&bag.file), Source::from(bag.source_content.clone())));
        }
    }

    pub fn has_reports() -> bool {
        !Self::global_mut().reports.is_empty()
    }

    pub fn has_errors() -> bool {
        Self::global_mut().has_errors
    }

    pub fn mark_errors() {
        Self::global_mut().has_errors = true;
    }

    pub fn clear_errors() {
        Self::global_mut().reports.clear();
    }

    pub fn file() -> Arc<MPath> {
        Arc::clone(&Self::global_mut().file)
    }
}

pub static REPORTS_BAG: Lazy<Mutex<ReportsBag>> = Lazy::new(|| {
    let dummy_path = Arc::new(MPath::new());
    let dummy_source = Arc::<str>::from("");
    Mutex::new(ReportsBag::new(dummy_path, dummy_source))
});

#[macro_export]
macro_rules! report {
    (
        kind: $kind:expr,
        message: $message:expr
        $(, labels: {
            $(
                $label_key:expr => $label_msg:expr => $label_color:expr
            ),* $(,)?
        })?
        $(, label_vec: $label_vec:expr $(=> $vec_color:expr)? )?
        $(, notes: [$($note:expr),* $(,)?])?
        $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut report = Report::build($kind, (ReportsBag::file(), Span::default()))
            .with_message($message);

        $(
            $(
                let label = Label::new((ReportsBag::file(), $label_key))
                    .with_message($label_msg)
                    .with_color($label_color);
                report = report.with_label(label);
            )*
        )?

        $(
            for (span, msg) in $label_vec {
                let label = Label::new((ReportsBag::file(), span.clone()))
                    .with_message(msg)
                    .with_color(report!(@vec_color $($vec_color)?));
                report = report.with_label(label);
            }
        )?

        $(
            $(
                report = report.with_note($note);
            )*
        )?

        if $kind == ReportKind::Error {
            ReportsBag::mark_errors();
        }
        report.finish()
    }};

    (@vec_color $color:expr) => { $color };
    (@vec_color) => { ::ariadne::Color::Red };
}
