use crate::types::{self, NameSpecification, Type};


pub fn mangle_name(name_spec: &NameSpecification) -> String {
    // For now, we will just mangle by prefixing with the package name, but in the future we will need to mangle generics and stuff too
    format!("_L{}_{}", name_spec.package, name_spec.name)
}

pub fn mangle_method_name(id: &str, typ: &Type) -> String {
    // For now, we will just mangle by prefixing with the package name, but in the future we will need to mangle generics and stuff too
    format!("_L_ms_{}_{}", types::name(typ), id)
}