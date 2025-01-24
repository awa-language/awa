#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuiltInFunctions {
    pub type_: BuiltInFunctionsTypes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInFunctionsTypes {
    Print,
    Println,
    Append,
}

impl BuiltInFunctions {
    #[must_use]
    pub fn get_name(&self) -> String {
        match &self.type_ {
            BuiltInFunctionsTypes::Print => "print".to_owned(),
            BuiltInFunctionsTypes::Println => "println".to_owned(),
            BuiltInFunctionsTypes::Append => "append".to_owned(),
        }
    }
}
