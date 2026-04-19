use std::collections::HashMap;

use crate::analyzer::{
    field::FieldModel,
    registry::LibClassId,
    signature::{MethodSignature, ParamsSignature},
};

pub struct LibClassModel {
    pub id: LibClassId,
    pub parent: LibClassId,
    pub constructors: Vec<ParamsSignature>,
    pub methods: HashMap<&'static str, Vec<MethodSignature>>,
    pub fields: HashMap<&'static str, FieldModel>,
}

pub fn models() -> HashMap<&'static str, LibClassModel> {
    let mut hashmap = HashMap::new();
    hashmap.insert(
        "Class",
        LibClassModel {
            id: LibClassId::Class,
            parent: LibClassId::Class,
            constructors: vec![ParamsSignature::new(vec![])],
            methods: HashMap::new(),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "AnyRef",
        LibClassModel {
            id: LibClassId::AnyRef,
            parent: LibClassId::Class,
            constructors: vec![ParamsSignature::empty()],
            methods: HashMap::from([(
                "ToString",
                vec![MethodSignature {
                    params: ParamsSignature::empty(),
                    return_type: Some(LibClassId::String.into()),
                }],
            )]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "AnyValue",
        LibClassModel {
            id: LibClassId::AnyValue,
            parent: LibClassId::Class,
            constructors: vec![ParamsSignature::empty()],
            methods: HashMap::from([(
                "ToString",
                vec![MethodSignature {
                    params: ParamsSignature::empty(),
                    return_type: Some(LibClassId::String.into()),
                }],
            )]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Integer",
        LibClassModel {
            id: LibClassId::Integer,
            parent: LibClassId::AnyValue,
            constructors: vec![
                ParamsSignature::new(vec![LibClassId::Integer.into()]),
                ParamsSignature::new(vec![LibClassId::Real.into()]),
            ],
            methods: HashMap::from([
                (
                    "ToReal",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Real.into()),
                    }],
                ),
                (
                    "ToBoolean",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Boolean.into()),
                    }],
                ),
                (
                    "ToChar",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Char.into()),
                    }],
                ),
                (
                    "UnaryMinus",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Integer.into()),
                    }],
                ),
                (
                    "Plus",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Integer.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Minus",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Integer.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Mult",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Integer.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Div",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Integer.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Rem",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                        return_type: Some(LibClassId::Integer.into()),
                    }],
                ),
                (
                    "Less",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "LessEqual",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "Greater",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "GreaterEqual",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "Equal",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Real",
        LibClassModel {
            id: LibClassId::Real,
            parent: LibClassId::AnyValue,
            constructors: vec![
                ParamsSignature::new(vec![LibClassId::Real.into()]),
                ParamsSignature::new(vec![LibClassId::Integer.into()]),
            ],
            methods: HashMap::from([
                (
                    "ToInteger",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Integer.into()),
                    }],
                ),
                (
                    "UnaryMinus",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Real.into()),
                    }],
                ),
                (
                    "Plus",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Minus",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Mult",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Div",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Real.into()),
                        },
                    ],
                ),
                (
                    "Rem",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                        return_type: Some(LibClassId::Real.into()),
                    }],
                ),
                (
                    "Less",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "LessEqual",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "Greater",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "GreaterEqual",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
                (
                    "Equal",
                    vec![
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                        },
                    ],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Boolean",
        LibClassModel {
            id: LibClassId::Boolean,
            parent: LibClassId::AnyValue,
            constructors: vec![ParamsSignature::empty()],
            methods: HashMap::from([
                (
                    "toInteger",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Integer.into()),
                    }],
                ),
                (
                    "Or",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                        return_type: Some(LibClassId::Boolean.into()),
                    }],
                ),
                (
                    "And",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                        return_type: Some(LibClassId::Boolean.into()),
                    }],
                ),
                (
                    "Xor",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                        return_type: Some(LibClassId::Boolean.into()),
                    }],
                ),
                (
                    "Not",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Boolean.into()),
                    }],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Array",
        LibClassModel {
            id: LibClassId::Array,
            parent: LibClassId::AnyRef,
            constructors: vec![ParamsSignature::new(vec![LibClassId::Integer.into()])],
            methods: HashMap::from([
                (
                    "toList",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::List.into()),
                    }],
                ),
                (
                    "Length",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::Integer.into()),
                    }],
                ),
                (
                    "Get",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                        return_type: Some(LibClassId::AnyRef.into()),
                    }],
                ),
                (
                    "Set",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![
                            LibClassId::Integer.into(),
                            LibClassId::AnyRef.into(),
                        ]),
                        return_type: None,
                    }],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "List",
        LibClassModel {
            id: LibClassId::List,
            parent: LibClassId::AnyRef,
            constructors: vec![
                ParamsSignature::empty(),
                ParamsSignature::new(vec![LibClassId::AnyRef.into()]),
                ParamsSignature::new(vec![LibClassId::AnyRef.into(), LibClassId::Integer.into()]),
            ],
            methods: HashMap::from([
                (
                    "Append",
                    vec![MethodSignature {
                        params: ParamsSignature::new(vec![LibClassId::AnyRef.into()]),
                        return_type: Some(LibClassId::List.into()),
                    }],
                ),
                (
                    "Head",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::AnyRef.into()),
                    }],
                ),
                (
                    "Tail",
                    vec![MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::List.into()),
                    }],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    //TODO char+string
    hashmap
}
