pub mod bool_t;
pub mod f16_t;
pub mod f32_t;
pub mod f64_t;
pub mod i128_t;
pub mod i16_t;
pub mod i32_t;
pub mod i64_t;
pub mod i8_t;
pub mod ibig_t;
pub mod isize_t;
pub mod macros;
pub mod proc_t;
pub mod str_t;
pub mod type_t;
pub mod u128_t;
pub mod u16_t;
pub mod u32_t;
pub mod u64_t;
pub mod u8_t;
pub mod ubig_t;
pub mod unit_t;
pub mod usize_t;

use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

use itertools::Itertools;
use smol_str::SmolStr;

use crate::{
    interpreter::interpret_expr,
    types::{
        element::Argument,
        typeobj::{
            type_t::{TYPE_T, TYPE_T_ELE},
            unit_t::{UNIT_T, UNIT_T_ELE},
        },
    },
    Element, InterpreterData, Print, Value, ZyxtError,
};

#[derive(Clone, PartialEq)]
pub struct TypeDefinition<T: Clone + PartialEq + Debug> {
    // class, struct, (anything that implements a Type). Is of type <type> (Typedef)
    pub inst_name: Option<SmolStr>, // TODO inheritance
    pub name: Option<SmolStr>,
    pub generics: Vec<Argument>,
    pub implementations: HashMap<SmolStr, T>,
    pub inst_fields: HashMap<SmolStr, (Box<Type<T>>, Option<T>)>,
}

#[derive(Clone, PartialEq)]
pub struct TypeInstance<T: Clone + PartialEq + Debug> {
    // str, bool, cpx<int> etc. Is of type Typedef
    pub name: Option<SmolStr>,
    pub type_args: Vec<Type<T>>,
    pub implementation: TypeDefinition<T>,
}

#[derive(Clone, PartialEq)]
pub enum Type<T: Clone + PartialEq + Debug> {
    Instance(TypeInstance<T>),
    Definition(TypeDefinition<T>),
    Any,
    Return(Box<Type<T>>),
}
impl<T: Clone + PartialEq + Debug> Debug for TypeInstance<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (implementation: {:?})", self, self.implementation)
    }
}
impl<T: Clone + PartialEq + Debug> Debug for TypeDefinition<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} for {} (implementations: {{{}}}; fields: {{{}}})",
            self,
            self.inst_name.unwrap_or_else(|| "{unknown}".into()),
            self.implementations.iter().map(|(k, _)| k).join(", "),
            self.inst_fields.iter().map(|(k, _)| k).join(", ")
        )
    }
}
impl<T: Clone + PartialEq + Debug> Debug for Type<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Instance(inst) => {
                write!(f, "{inst:?}")
            }
            Type::Definition(def) => {
                write!(f, "{def:?}")
            }
            Type::Any => write!(f, "_any"),
            Type::Return(t) => <Self as Debug>::fmt(t, f),
        }
    }
}
impl<T: Clone + PartialEq + Debug> Display for TypeDefinition<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.name.to_owned().unwrap_or_else(|| "{unknown}".into())
        )
    }
}
impl<T: Clone + PartialEq + Debug> Display for TypeInstance<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}>",
            self.name.as_ref().unwrap_or(&"{unknown}".into()),
            self.type_args
                .iter()
                .map(|arg| format!("{}", arg))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl<T: Clone + PartialEq + Debug> Display for Type<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Instance(inst) => inst.to_string(),
                Type::Definition(def) => def.to_string(),
                Type::Any => "_any".into(),
                Type::Return(ty) => ty.to_string(),
            }
        )
    }
}

impl TypeDefinition<Element> {
    pub fn get_instance(&self) -> Type<Element> {
        if *self == TYPE_T.as_type_element() {
            Type::Definition(TYPE_T.as_type_element())
        } else {
            Type::Instance(TypeInstance {
                name: self.inst_name.to_owned(),
                type_args: vec![],
                implementation: self.to_owned(),
            })
        }
    }
}
impl TypeDefinition<Value> {
    pub fn get_instance(&self) -> Type<Value> {
        if *self == *TYPE_T {
            Type::Definition(TYPE_T.to_owned())
        } else {
            Type::Instance(TypeInstance {
                name: self.inst_name.to_owned(),
                type_args: vec![],
                implementation: self.to_owned(),
            })
        }
    }
}

impl TypeDefinition<Element> {
    pub fn as_type_value(
        &self,
        i_data: &mut InterpreterData<Value, impl Print>,
    ) -> Result<TypeDefinition<Value>, ZyxtError> {
        Ok(TypeDefinition {
            inst_name: self.inst_name.to_owned(),
            name: self.name.to_owned(),
            generics: self.generics.to_owned(),
            implementations: self
                .implementations
                .iter()
                .map(|(k, v)| Ok((k.to_owned(), interpret_expr(v, i_data)?)))
                .collect::<Result<HashMap<_, _>, _>>()?,
            inst_fields: self
                .inst_fields
                .iter()
                .map(|(k, (v1, v2))| {
                    Ok((
                        k.to_owned(),
                        (
                            Box::new(v1.as_type_value(i_data)?),
                            v2.to_owned()
                                .map(|v2| interpret_expr(&v2, i_data))
                                .transpose()?,
                        ),
                    ))
                })
                .collect::<Result<HashMap<_, _>, _>>()?,
        })
    }
}
impl TypeInstance<Element> {
    pub fn as_type_value(
        &self,
        i_data: &mut InterpreterData<Value, impl Print>,
    ) -> Result<TypeInstance<Value>, ZyxtError> {
        Ok(TypeInstance {
            name: self.name.to_owned(),
            type_args: self
                .type_args
                .iter()
                .map(|a| a.as_type_value(i_data))
                .collect::<Result<Vec<_>, _>>()?,
            implementation: self.implementation.as_type_value(i_data)?,
        })
    }
}
impl TypeInstance<Value> {
    pub fn as_type_element(&self) -> TypeInstance<Element> {
        TypeInstance {
            name: self.name.to_owned(),
            type_args: self.type_args.iter().map(|a| a.as_type_element()).collect(),
            implementation: self.implementation.as_type_element(),
        }
    }
}
impl TypeDefinition<Value> {
    pub fn as_type_element(&self) -> TypeDefinition<Element> {
        TypeDefinition {
            inst_name: self.inst_name.to_owned(),
            name: self.name.to_owned(),
            generics: self.generics.to_owned(),
            implementations: self
                .implementations
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_owned(),
                        Element::Literal {
                            position: Default::default(),
                            raw: "".into(),
                            content: v.to_owned(),
                        },
                    )
                })
                .collect(),
            inst_fields: self
                .inst_fields
                .iter()
                .map(|(k, (v1, v2))| {
                    (
                        k.to_owned(),
                        (
                            Box::new(v1.as_type_element()),
                            v2.to_owned().map(|v2| Element::Literal {
                                position: Default::default(),
                                raw: "".into(),
                                content: v2,
                            }),
                        ),
                    )
                })
                .collect(),
        }
    }
}

impl Type<Element> {
    pub fn as_literal(&self) -> Element {
        Element::Literal {
            position: Default::default(),
            raw: "".into(),
            content: Value::PreType(self.to_owned()),
        }
    }
    pub fn implementation(&self) -> &TypeDefinition<Element> {
        match &self {
            Type::Instance(TypeInstance { implementation, .. }) => implementation,
            Type::Definition { .. } => &TYPE_T_ELE,
            Type::Any => &UNIT_T_ELE,
            Type::Return(ty) => ty.implementation(),
        }
    }
    pub fn as_type_value(
        &self,
        i_data: &mut InterpreterData<Value, impl Print>,
    ) -> Result<Type<Value>, ZyxtError> {
        Ok(match &self {
            Type::Instance(inst) => Type::Instance(inst.as_type_value(i_data)?),
            Type::Definition(def) => Type::Definition(def.as_type_value(i_data)?),
            Type::Any => Type::Any,
            Type::Return(t) => Type::Return(Box::new(t.as_type_value(i_data)?)),
        })
    }
}

impl Type<Value> {
    pub fn implementation(&self) -> &TypeDefinition<Value> {
        match &self {
            Type::Instance(TypeInstance { implementation, .. }) => implementation,
            Type::Definition { .. } => &*TYPE_T,
            Type::Any => &*UNIT_T,
            Type::Return(ty) => ty.implementation(),
        }
    }
    pub fn as_type_element(&self) -> Type<Element> {
        match &self {
            Type::Instance(inst) => Type::Instance(inst.as_type_element()),
            Type::Definition(def) => Type::Definition(def.as_type_element()),
            Type::Any => Type::Any,
            Type::Return(t) => Type::Return(Box::new(t.as_type_element())),
        }
    }
}