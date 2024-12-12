use ecow::EcoString;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StringSpan {
    location: super::location::Location,
    value: EcoString,
}
