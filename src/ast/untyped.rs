use uper::location::Location;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UntypedExpr {
    Int32 {
        location : Location,
        value : EcoString,
    },

    Int64 {
        location : Location,
        value : EcoString,
    },

    Float32{ 
        location : Location,
        value : EcoString,
    },

    Float64{ 
        location : Location,
        value : EcoString,
    },

}
