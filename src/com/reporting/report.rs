use super::{Header, Label};
use codespan_reporting::diagnostic::{self, Diagnostic, LabelStyle, Severity};
use logos::Span;

pub struct Report {
    pub header: Header,
    pub severity: diagnostic::Severity,
    pub labels: Vec<(Label, Span, diagnostic::LabelStyle)>,
}

impl Report {
    pub fn new(severity: Severity, header: Header) -> Self {
        Self {
            header,
            severity,
            labels: Vec::new(),
        }
    }

    pub fn error(header: Header) -> Self {
        Self {
            header,
            severity: Severity::Error,
            labels: Vec::new(),
        }
    }

    pub fn with_primary_label(mut self, label: Label, span: Span) -> Self {
        self.labels.push((label, span, LabelStyle::Primary));
        self
    }

    pub fn with_secondary_label(mut self, label: Label, span: Span) -> Self {
        self.labels.push((label, span, LabelStyle::Secondary));
        self
    }

    pub fn to_diagnostic(&self, file: usize) -> Diagnostic<usize> {
        use diagnostic::Diagnostic as D;
        use diagnostic::Label as L;
        return D::new(self.severity)
            .with_code(self.header.name())
            .with_message(self.header.msg())
            .with_labels(
                self.labels
                    .iter()
                    .map(|(label, span, style)| {
                        L::new(*style, file, span.clone()).with_message(label.msg())
                    })
                    .collect(),
            );
    }
}
