use crate::MPath;
use ariadne::{Report, Source};
use std::borrow::Cow;
use std::ops::Range;

pub type MReport<'a> = Report<'a, (&'a MPath, Range<usize>)>;

#[derive(Debug)]
pub struct ReportsBag<'a> {
    reports: Vec<MReport<'a>>,
    file_name: &'a MPath,
    source_content: &'a str,
}

impl<'a> ReportsBag<'a> {
    pub fn new(file_name: &'a MPath, source_content: &'a str) -> Self {
        Self { reports: Vec::new(), file_name, source_content }
    }

    pub fn add(&mut self, report: MReport<'a>) {
        self.reports.push(report);
    }

    pub fn print(&self) {
        self.reports.iter().for_each(|report| {
            report.print((self.file_name, Source::from(self.source_content))).unwrap();
        })
    }

    pub fn extend(&mut self, temp: TemporaryBag<'a>) {
        self.reports.extend(temp.reports);
    }
}

#[derive(Debug)]
// ReportsBag will later extend this
pub struct TemporaryBag<'a> {
    reports: Vec<MReport<'a>>,
}

impl<'a> TemporaryBag<'a> {
    pub fn new() -> Self {
        Self { reports: Vec::new() }
    }

    pub fn add(&mut self, report: MReport<'a>) {
        self.reports.push(report);
    }
}

#[macro_export]
macro_rules! error_report {
    (
        file: $file:expr,
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
        let mut report = Report::build(ReportKind::Error, ($file, $span))
            .with_message($message);

        $(
            $(
                let label = Label::new(($file, $label_span))
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
