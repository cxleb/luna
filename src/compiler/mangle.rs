use crate::types::{self, NameSpecification, Type};

fn sanitise_symbol_part(input: &str) -> String {
    input
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub fn mangle_name(name_spec: &NameSpecification) -> String {
    format!(
        "_L{}_{}_{}",
        sanitise_symbol_part(&name_spec.package),
        sanitise_symbol_part(&name_spec.file),
        sanitise_symbol_part(&name_spec.name)
    )
}

pub fn mangle_method_name(id: &str, typ: &Type) -> String {
    // For now, we will just mangle by prefixing with the package name, but in the future we will need to mangle generics and stuff too
    format!("_L_ms_{}_{}", types::name(typ), id)
}
