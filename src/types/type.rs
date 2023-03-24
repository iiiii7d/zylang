use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    ops::Deref,
    sync::Arc,
};

use itertools::Either;
use once_cell::sync::OnceCell;
use smol_str::SmolStr;

use crate::{ast::Ident, primitives::ANY_T_VAL, types::value::Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Any,
    Type {
        name: Option<Ident>,
        namespace: HashMap<SmolStr, LazyType<Value>>,
        fields: HashMap<SmolStr, Arc<Type>>,
        type_args: Vec<(SmolStr, Arc<Type>)>,
    },
    Generic {
        type_args: Vec<(SmolStr, Either<Value, Arc<Type>>)>,
        base: Arc<Type>,
    },
}

#[derive(Clone)]
pub struct LazyType<T: Clone + Debug> {
    pub data: Option<T>,
    ty: OnceCell<Arc<Type>>,
    f: Arc<dyn Fn(&Option<T>) -> Arc<Type> + Send + Sync>,
}
impl<T: Clone + Debug> PartialEq for LazyType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}
impl<T: Clone + Debug> Debug for LazyType<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}
impl<T: Clone + Debug> Deref for LazyType<T> {
    type Target = Arc<Type>;
    fn deref(&self) -> &Self::Target {
        self.ty.get_or_init(|| (self.f)(&self.data))
    }
}
impl<T: Clone + Debug + 'static> LazyType<T> {
    pub fn new_lazy(data: T, f: fn(&T) -> Arc<Type>) -> Self {
        Self {
            data: Some(data),
            f: Arc::new(move |v| f(v.as_ref().unwrap())),
            ty: OnceCell::new(),
        }
    }
}

impl<T: Clone + Debug> From<Arc<Type>> for LazyType<T> {
    fn from(ty: Arc<Type>) -> Self {
        Self {
            data: None,
            f: Arc::new(move |_| Arc::clone(&ty)),
            ty: OnceCell::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueType {
    Any,
    Type {
        name: Option<Ident>,
        namespace: HashMap<SmolStr, Value>,
        fields: HashMap<SmolStr, Arc<Type>>,
        type_args: Vec<(SmolStr, Value)>,
    },
}

impl From<ValueType> for Type {
    fn from(_value: ValueType) -> Self {
        todo!()
    }
}

impl Display for Type {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for ValueType {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuiltinType {
    pub name: Option<Ident>,
    pub namespace: HashMap<SmolStr, Value>,
    pub fields: HashMap<SmolStr, Arc<Type>>,
    pub type_args: Vec<(SmolStr, Arc<Type>)>,
}
impl From<BuiltinType> for Type {
    fn from(value: BuiltinType) -> Self {
        Self::Type {
            name: value.name,
            namespace: value
                .namespace
                .into_iter()
                .map(|(k, v)| (k, LazyType::new_lazy(v, Value::ty)))
                .collect(),
            fields: value.fields,
            type_args: value.type_args,
        }
    }
}
impl From<BuiltinType> for ValueType {
    fn from(value: BuiltinType) -> Self {
        Self::Type {
            name: value.name,
            namespace: value.namespace,
            fields: value.fields,
            type_args: value
                .type_args
                .into_iter()
                .map(|(k, _)| (k, Value::Type(Arc::clone(&ANY_T_VAL))))
                .collect(),
        }
    }
}
