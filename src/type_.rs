use std::sync::Arc;

use vec1::Vec1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Func {
        arguments: Vec1<Arc<Type>>,
        return_annotation: Arc<Type>,
    },
    Var {
        id: u64,
    },
}
