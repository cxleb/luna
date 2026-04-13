use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

// Define a static global counter
static TYPE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn get_next_id() -> usize {
    // Increment and return the new value safely
    TYPE_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct NameSpecification {
    pub package: String,
    pub file: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub returns: Vec<Type>,
}

#[derive(Debug)]
pub struct StructType {
    pub spec: NameSpecification,
    pub fields: RwLock<Vec<(String, Type)>>,
}

#[derive(Debug)]
pub struct EnumType {
    pub spec: NameSpecification,
    pub variants: RwLock<Vec<(String, Vec<Type>)>>,
}

#[derive(Debug)]
pub struct InterfaceType {
    pub spec: NameSpecification,
    pub methods: RwLock<Vec<(String, FunctionType)>>,
}

#[derive(Debug)]
pub enum TypeKind {
    Bad,
    Integer,
    Byte,
    Number,
    String,
    Bool,
    UnknownReference, // An internal detail before generics is correctly implemented
    Array(Type),
    Struct(StructType),
    Enum(EnumType),
    Function(FunctionType),
    Interface(InterfaceType),
}

#[derive(Debug)]
pub struct Inner {
    pub hash: usize,
    pub kind: TypeKind,
    pub method_set: RwLock<Vec<(String, FunctionType)>>,
}

#[derive(Debug)]
pub struct Type {
    pub inner: Arc<Inner>,
}

impl Type {
    pub fn kind(&self) -> &TypeKind {
        &self.inner.kind
    }

    pub fn add_method(&self, name: &str, f: FunctionType) {
        self.inner
            .method_set
            .write()
            .unwrap()
            .push((name.into(), f));
    }

    pub fn get_method(&self, name: &str) -> Option<FunctionType> {
        if let TypeKind::Interface(interface) = &self.inner.kind {
            if let Some((_, method)) = interface
                .methods
                .read()
                .unwrap()
                .iter()
                .find(|(n, _)| n == name)
            {
                return Some(method.clone());
            }
        }
        Some(
            self.inner
                .method_set
                .read()
                .unwrap()
                .iter()
                .find(|(n, _)| n == name)?
                .1
                .clone(),
        )
    }
}

impl Clone for Type {
    fn clone(&self) -> Self {
        Type {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        bad()
    }
}

impl Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash.hash(state);
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.inner.hash == other.inner.hash
    }
}

impl Eq for Type {}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonResult {
    Same,
    Upcastable,
    Incompatible,
}

pub fn interface_assignable(i: &InterfaceType, typ: &Type) -> bool {
    if i.methods.read().unwrap().is_empty() {
        return true;
    }

    for f in i.methods.read().unwrap().iter() {
        if let Some(method) = typ.get_method(&f.0) {
            if f.1 != method {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

/// Compares two types
/// If one can be an interface, the interface should be `a`
pub fn compare(a: &Type, b: &Type) -> ComparisonResult {
    if a == b {
        return ComparisonResult::Same;
    }

    if let TypeKind::Array(a_elem_typ) = a.kind() {
        if let TypeKind::Array(b_elem_typ) = b.kind() {
            let r = compare(a_elem_typ, b_elem_typ);
            if r == ComparisonResult::Same {
                return r;
            }
        }
    }

    if let TypeKind::Interface(i) = a.kind() {
        if interface_assignable(i, b) {
            return ComparisonResult::Upcastable;
        }
    }

    return ComparisonResult::Incompatible;
}

/// Is type numeric (integer or number)
pub fn is_numeric(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Integer | TypeKind::Byte | TypeKind::Number)
}

pub fn is_byte(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Byte)
}

pub fn is_bad(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Bad)
}

pub fn is_unknown_reference(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::UnknownReference)
}

pub fn is_number(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Number)
}

pub fn is_integer(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Integer)
}

pub fn is_array(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Array(_))
}

pub fn is_bool(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Bool)
}

pub fn is_string(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::String)
}

pub fn is_struct(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Struct(_))
}

pub fn is_enum(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Enum(_))
}

pub fn is_function(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Function(_))
}

pub fn is_reference(ty: &Type) -> bool {
    matches!(
        ty.inner.kind,
        TypeKind::UnknownReference
            | TypeKind::Struct(_)
            | TypeKind::Array(_)
            | TypeKind::Interface(_)
    )
}

pub fn is_interface(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Interface(_))
}

pub fn clone_struct_fields(ty: &Type) -> Vec<(String, Type)> {
    if let TypeKind::Struct(struct_type) = &ty.inner.kind {
        struct_type.fields.read().unwrap().clone()
    } else {
        panic!("Type is not a struct");
    }
}

pub fn get_max_enum_values(ty: &Type) -> usize {
    if let TypeKind::Enum(enum_type) = &ty.inner.kind {
        enum_type
            .variants
            .read()
            .unwrap()
            .iter()
            .map(|(_, values)| values.len())
            .max()
            .unwrap_or(0)
    } else {
        panic!("Type is not an enum");
    }
}

pub fn get_interface_func_index(ty: &Type, id: &str) -> usize {
    if let TypeKind::Interface(interface) = ty.kind() {
        interface
            .methods
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .filter(|(_, m)| m.0 == *id)
            .map(|(i, _)| i)
            .next()
            .unwrap_or_else(|| panic!("Function not found in interface"))
    } else {
        panic!("Type is not an interface");
    }
}

pub fn create_type(kind: TypeKind) -> Type {
    Type {
        inner: Arc::new(Inner {
            kind,
            hash: get_next_id(),
            method_set: RwLock::new(Vec::new()),
        }),
    }
}

pub fn name(typ: &Type) -> String {
    match &typ.inner.kind {
        TypeKind::Bad => "bad".into(),
        TypeKind::Integer => "integer".into(),
        TypeKind::Byte => "byte".into(),
        TypeKind::Number => "number".into(),
        TypeKind::String => "string".into(),
        TypeKind::Bool => "bool".into(),
        TypeKind::UnknownReference => "unknown_reference".into(),
        TypeKind::Array(element_type) => format!("[]{}", name(element_type)),
        TypeKind::Struct(struct_type) => format!(
            "{}:{}:{}",
            struct_type.spec.package, struct_type.spec.file, struct_type.spec.name
        ),
        TypeKind::Enum(enum_type) => format!(
            "{}:{}:{}",
            enum_type.spec.package, enum_type.spec.file, enum_type.spec.name
        ),
        TypeKind::Function(func_type) => format!(
            "fn({}) -> ({})",
            func_type
                .params
                .iter()
                .map(|t| name(t))
                .collect::<Vec<_>>()
                .join(", "),
            func_type
                .returns
                .iter()
                .map(|t| name(t))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeKind::Interface(interface_type) => format!(
            "{}:{}:{}",
            interface_type.spec.package, interface_type.spec.file, interface_type.spec.name
        ),
    }
}

pub fn bad() -> Type {
    static UNKNOWN_TYPE: OnceLock<Type> = OnceLock::new();
    UNKNOWN_TYPE
        .get_or_init(|| create_type(TypeKind::Bad))
        .clone()
}

pub fn integer() -> Type {
    static INTEGER_TYPE: OnceLock<Type> = OnceLock::new();
    INTEGER_TYPE
        .get_or_init(|| create_type(TypeKind::Integer))
        .clone()
}

pub fn byte() -> Type {
    static BYTE_TYPE: OnceLock<Type> = OnceLock::new();
    BYTE_TYPE
        .get_or_init(|| create_type(TypeKind::Byte))
        .clone()
}

pub fn number() -> Type {
    static NUMBER_TYPE: OnceLock<Type> = OnceLock::new();
    NUMBER_TYPE
        .get_or_init(|| create_type(TypeKind::Number))
        .clone()
}

pub fn bool() -> Type {
    static BOOL_TYPE: OnceLock<Type> = OnceLock::new();
    BOOL_TYPE
        .get_or_init(|| create_type(TypeKind::Bool))
        .clone()
}

pub fn string() -> Type {
    static STRING_TYPE: OnceLock<Type> = OnceLock::new();
    STRING_TYPE
        .get_or_init(|| create_type(TypeKind::String))
        .clone()
}

pub fn unknown_reference() -> Type {
    static UNKNOWN_REFERENCE_TYPE: OnceLock<Type> = OnceLock::new();
    UNKNOWN_REFERENCE_TYPE
        .get_or_init(|| create_type(TypeKind::UnknownReference))
        .clone()
}

pub fn array(element_type: Type) -> Type {
    // memoize the array so that hash is the same for the given element type
    // this is not a performance thing
    static ARRAY_TYPES: OnceLock<RwLock<HashMap<usize, Type>>> = OnceLock::new();
    let array_types = ARRAY_TYPES.get_or_init(|| RwLock::new(HashMap::new()));
    let mut array_types = array_types.write().unwrap();
    if let Some(array_type) = array_types.get(&element_type.inner.hash) {
        return array_type.clone();
    }
    let array_type = create_type(TypeKind::Array(element_type.clone()));
    array_types.insert(element_type.inner.hash, array_type.clone());
    array_type
}

pub fn struct_type(spec: NameSpecification, fields: Vec<(String, Type)>) -> Type {
    create_type(TypeKind::Struct(StructType {
        spec,
        fields: RwLock::new(fields),
    }))
}

pub fn enum_type(spec: NameSpecification, variants: Vec<(String, Vec<Type>)>) -> Type {
    create_type(TypeKind::Enum(EnumType {
        spec,
        variants: RwLock::new(variants),
    }))
}

pub fn function_type(params: Vec<Type>, returns: Vec<Type>) -> Type {
    create_type(TypeKind::Function(FunctionType { params, returns }))
}

pub fn interface_type(spec: NameSpecification, methods: Vec<(String, FunctionType)>) -> Type {
    create_type(TypeKind::Interface(InterfaceType {
        spec,
        methods: RwLock::new(methods),
    }))
}
