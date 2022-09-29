use std::collections::HashMap;

use half::f16;
use lazy_static::lazy_static;
use smol_str::SmolStr;

use crate::{
    arith_opr_num, binary, comp_opr_num, concat_vals, get_param, typecast_int, typecast_to_type,
    types::{
        typeobj::{
            bool_t::BOOL_T, f16_t::F16_T, f32_t::F32_T, f64_t::F64_T, i128_t::I128_T, i16_t::I16_T,
            i32_t::I32_T, i64_t::I64_T, i8_t::I8_T, ibig_t::IBIG_T, isize_t::ISIZE_T, str_t::STR_T,
            type_t::TYPE_T, u128_t::U128_T, u32_t::U32_T, u64_t::U64_T, u8_t::U8_T, ubig_t::UBIG_T,
            usize_t::USIZE_T, TypeDefinition,
        },
        value::{Proc, Value},
    },
    unary, Type,
};

fn u16_t() -> HashMap<SmolStr, Value> {
    let mut h = HashMap::new();
    h.insert("_default", Value::U16(0));
    concat_vals!(h, U16_T);
    unary!(h, signed default U16_T U16);
    arith_opr_num!(h, default U16_T U16);
    comp_opr_num!(h, default U16_T U16);

    let typecast = |x: &Vec<Value>| {
        Some(match get_param!(x, 1, Type) {
            p if p == *TYPE_T => typecast_to_type!(U16_T),
            p if p == *STR_T => typecast_int!(U16 => str, x),
            p if p == *BOOL_T => typecast_int!(U16 => bool, x),
            p if p == *I8_T => typecast_int!(U16 => I8, x),
            p if p == *I16_T => typecast_int!(U16 => I16, x),
            p if p == *I32_T => typecast_int!(U16 => I32, x),
            p if p == *I64_T => typecast_int!(U16 => I64, x),
            p if p == *I128_T => typecast_int!(U16 => I128, x),
            p if p == *ISIZE_T => typecast_int!(U16 => Isize, x),
            p if p == *IBIG_T => typecast_int!(U16 => Ibig, x),
            p if p == *U8_T => typecast_int!(U16 => U8, x),
            p if p == *U16_T => x[0].to_owned(),
            p if p == *U32_T => typecast_int!(U16 => U32, x),
            p if p == *U64_T => typecast_int!(U16 => U64, x),
            p if p == *U128_T => typecast_int!(U16 => U128, x),
            p if p == *USIZE_T => typecast_int!(U16 => Usize, x),
            p if p == *UBIG_T => typecast_int!(U16 => Ubig, x),
            p if p == *F16_T => typecast_int!(U16 => f16, x),
            p if p == *F32_T => typecast_int!(U16 => f32, x),
            p if p == *F64_T => typecast_int!(U16 => f64, x),
            _ => return None,
        })
    };
    binary!(h, U16_T, "_typecast", [TYPE_T], Type::Any, typecast);

    h.drain().map(|(k, v)| (k.into(), v)).collect()
}

lazy_static! {
    pub static ref U16_T: TypeDefinition<Value> = TypeDefinition {
        name: Some("{builtin u16}".into()),
        inst_name: Some("u16".into()),
        generics: vec![],
        implementations: u16_t(),
        inst_fields: HashMap::new(),
    };
}