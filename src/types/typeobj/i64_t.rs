use std::collections::HashMap;

use half::f16;
use once_cell::sync::Lazy;
use smol_str::SmolStr;

use crate::{
    arith_opr_num, binary, comp_opr_num, concat_vals, get_param, typecast_int, typecast_to_type,
    types::{
        typeobj::{
            bool_t::BOOL_T, f16_t::F16_T, f32_t::F32_T, f64_t::F64_T, i128_t::I128_T, i16_t::I16_T,
            i32_t::I32_T, i8_t::I8_T, ibig_t::IBIG_T, isize_t::ISIZE_T, str_t::STR_T,
            type_t::TYPE_T, u128_t::U128_T, u16_t::U16_T, u32_t::U32_T, u64_t::U64_T, u8_t::U8_T,
            ubig_t::UBIG_T, usize_t::USIZE_T, TypeDefinition,
        },
        value::{Proc, Value},
    },
    unary, Type,
};

fn i64_t() -> HashMap<SmolStr, Value> {
    let mut h = HashMap::new();
    h.insert("_default", Value::I64(0));
    concat_vals!(h, I64_T);
    unary!(h, signed default I64_T I64);
    arith_opr_num!(h, default I64_T I64);
    comp_opr_num!(h, default I64_T I64);

    let typecast = |x: &Vec<Value>| {
        Some(match get_param!(x, 1, Type) {
            p if p == TYPE_T.as_type() => typecast_to_type!(I64_T),
            p if p == STR_T.as_type() => typecast_int!(I64 => str, x),
            p if p == BOOL_T.as_type() => typecast_int!(I64 => bool, x),
            p if p == I8_T.as_type() => typecast_int!(I64 => I8, x),
            p if p == I16_T.as_type() => typecast_int!(I64 => I16, x),
            p if p == I32_T.as_type() => typecast_int!(I64 => I32, x),
            p if p == I64_T.as_type() => x[0].to_owned(),
            p if p == I128_T.as_type() => typecast_int!(I64 => I128, x),
            p if p == ISIZE_T.as_type() => typecast_int!(I64 => Isize, x),
            p if p == IBIG_T.as_type() => typecast_int!(I64 => Ibig, x),
            p if p == U8_T.as_type() => typecast_int!(I64 => U8, x),
            p if p == U16_T.as_type() => typecast_int!(I64 => U16, x),
            p if p == U32_T.as_type() => typecast_int!(I64 => U32, x),
            p if p == U64_T.as_type() => typecast_int!(I64 => U64, x),
            p if p == U128_T.as_type() => typecast_int!(I64 => U128, x),
            p if p == USIZE_T.as_type() => typecast_int!(I64 => Usize, x),
            p if p == UBIG_T.as_type() => typecast_int!(I64 => Ubig, x),
            p if p == F16_T.as_type() => typecast_int!(I64 => f16, x),
            p if p == F32_T.as_type() => typecast_int!(I64 => f32, x),
            p if p == F64_T.as_type() => typecast_int!(I64 => f64, x),
            _ => return None,
        })
    };
    binary!(
        h,
        I64_T.as_type(),
        "_typecast",
        [TYPE_T.as_type()],
        Type::Any,
        typecast
    );

    h.drain().map(|(k, v)| (k.into(), v)).collect()
}

pub static I64_T: Lazy<TypeDefinition<Value>> = Lazy::new(|| TypeDefinition {
    name: Some("{builtin i64}".into()),
    inst_name: Some("i64".into()),
    generics: vec![],
    implementations: i64_t(),
    inst_fields: HashMap::new(),
});