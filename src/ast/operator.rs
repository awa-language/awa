#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    And,
    Or,
    Equal,
    NotEqual,
    LessInt,
    LessEqualInt,
    LessFloat,
    LessEqualFloat,
    GreaterEqualInt,
    GreaterInt,
    GreaterEqualFloat,
    GreaterFloat,
    AdditionInt,
    AdditionFloat,
    SubtractionInt,
    SubtractionFloat,
    MultipicationInt,
    MultipicationFloat,
    DivisionInt,
    DivisionFloat,
    Modulo,
    Concatenation,
}

impl BinaryOperator {
    #[must_use]
    pub fn get_precedence(&self) -> u8 {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal | BinaryOperator::NotEqual => 3,
            BinaryOperator::LessInt
            | BinaryOperator::LessEqualInt
            | BinaryOperator::LessFloat
            | BinaryOperator::LessEqualFloat
            | BinaryOperator::GreaterEqualInt
            | BinaryOperator::GreaterInt
            | BinaryOperator::GreaterEqualFloat
            | BinaryOperator::GreaterFloat => 4,
            BinaryOperator::Concatenation => 5,
            BinaryOperator::AdditionInt
            | BinaryOperator::AdditionFloat
            | BinaryOperator::SubtractionInt
            | BinaryOperator::SubtractionFloat => 6,
            BinaryOperator::MultipicationInt
            | BinaryOperator::MultipicationFloat
            | BinaryOperator::DivisionInt
            | BinaryOperator::DivisionFloat
            | BinaryOperator::Modulo => 7,
        }
    }
}
