use super::{Header, Label, Note};
use crate::com::loc::Loc;
use codespan_reporting::diagnostic::{self, Diagnostic, LabelStyle, Severity};

pub struct Report {
    pub header: Header,
    pub severity: diagnostic::Severity,
    pub labels: Vec<(Label, Loc, diagnostic::LabelStyle)>,
    pub notes: Vec<Note>,
}

impl Report {
    pub fn error(header: Header) -> Self {
        Self {
            header,
            severity: Severity::Error,
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn warning(header: Header) -> Self {
        Self {
            header,
            severity: Severity::Warning,
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_primary_label(mut self, label: Label, loc: Loc) -> Self {
        self.labels.push((label, loc, LabelStyle::Primary));
        self
    }

    pub fn with_secondary_label(mut self, label: Label, loc: Loc) -> Self {
        self.labels.push((label, loc, LabelStyle::Secondary));
        self
    }

    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    pub fn to_diagnostic(&self) -> Diagnostic<usize> {
        use diagnostic::Diagnostic as D;
        use diagnostic::Label as L;
        return D::new(self.severity)
            .with_code(self.header.name())
            .with_message(self.header.msg())
            .with_labels(
                self.labels
                    .iter()
                    .map(|(label, loc, style)| {
                        L::new(*style, loc.file, loc.span).with_message(label.msg())
                    })
                    .collect(),
            )
            .with_notes(self.notes.iter().map(Note::msg).collect());
    }

    pub fn is_fatal(&self) -> bool {
        self.severity == Severity::Error
    }
}
