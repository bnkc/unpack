#![allow(dead_code)]
enum PyType {
    Int,
    Str,
    None,
    // ... other types as needed
}

pub fn convert_py_to_rs_types(pytype: &str) -> PyType {
    match pytype {
        "int" => PyType::Int,
        "str" => PyType::Str,
        "None" => PyType::None,
        _ => unimplemented!("{} is not a supported type", pytype),
    }
}
