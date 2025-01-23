#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuildInFunctions {
    pub type_: BuildInFunctionsTypes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildInFunctionsTypes {
    Print,
    Println,
    Append,
}

impl BuildInFunctions {
    #[must_use]
    pub fn get_description(&self) -> String {
        match &self.type_ {
            BuildInFunctionsTypes::Print => "print".to_owned(),
            BuildInFunctionsTypes::Println => "println".to_owned(),
            BuildInFunctionsTypes::Append => "append".to_owned(),
        }
    }
}
