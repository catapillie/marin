use super::{Header, Label};
use codespan_reporting::diagnostic::{self, Diagnostic, Severity};
use logos::Span;

pub struct Report<'src> {
    pub header: Header<'src>,
    pub severity: Severity,
    pub labels: Vec<(Label, Span)>,
}

impl<'src> Report<'src> {
    pub fn new(severity: Severity, header: Header<'src>) -> Self {
        Self {
            header,
            severity,
            labels: Vec::new(),
        }
    }

    pub fn error(header: Header<'src>) -> Self {
        Self {
            header,
            severity: Severity::Error,
            labels: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: Label, span: Span) -> Self {
        self.labels.push((label, span));
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
                    .map(|(label, span)| L::primary(file, span.clone()).with_message(label.msg()))
                    .collect(),
            );
    }
}
