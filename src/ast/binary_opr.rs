use std::{collections::HashMap, ops::Deref, sync::Arc};

use tracing::debug;

use crate::{
    ast::{Ast, AstData, Call, Literal, Member, Reconstruct},
    primitives::{BOOL_T, BOOL_T_VAL},
    types::{
        position::{GetSpan, Span},
        r#type::Type,
        sym_table::TypecheckSymTable,
        token::{AccessType, OprType},
    },
    InterpretSymTable, Value, ZResult,
};

#[derive(Clone, PartialEq, Debug)]
pub struct BinaryOpr {
    pub ty: OprType,
    pub opr_span: Option<Span>,
    pub operand1: Box<Ast>,
    pub operand2: Box<Ast>,
}
impl GetSpan for BinaryOpr {
    fn span(&self) -> Option<Span> {
        self.operand1
            .merge_span(&self.opr_span)
            .merge_span(&self.operand2)
    }
}

impl AstData for BinaryOpr {
    fn as_variant(&self) -> Ast {
        Ast::BinaryOpr(self.to_owned())
    }

    fn typecheck(&mut self, ty_symt: &mut TypecheckSymTable) -> ZResult<Arc<Type>> {
        self.operand1.typecheck(ty_symt)?;
        self.operand2.typecheck(ty_symt)?;
        match self.ty {
            OprType::And | OprType::Or => Ok(Arc::clone(&BOOL_T)),
            OprType::TypeCast => {
                if let Some(ty) = Some(ty_symt.get_type_from_ident(&self.operand2)?) {
                    Ok(ty)
                } else if let Ast::Literal(Literal {
                    content: Value::Type(ty),
                    ..
                }) = &*self.operand2
                {
                    Ok(ty.to_type())
                } else {
                    todo!()
                }
            }
            _ => unreachable!(),
        }
    }

    fn desugared(&self) -> ZResult<Ast> {
        Ok(match self.ty {
            OprType::And | OprType::Or => {
                debug!(span = ?self.span(), "Desugaring && / || operator");
                let mut new_self = self.to_owned();
                for operand in [&mut new_self.operand1, &mut new_self.operand2] {
                    *operand = Self {
                        ty: OprType::TypeCast,
                        opr_span: self.opr_span.to_owned(),
                        operand1: operand.desugared()?.into(),
                        operand2: Box::new(Value::Type(Arc::clone(&BOOL_T_VAL)).as_ast()),
                    }
                    .desugared()?
                    .into();
                }
                new_self.as_variant()
            }
            OprType::TypeCast => {
                debug!(span = ?self.span(), "Desugaring @ operator");
                let mut new_self = self.to_owned();
                new_self.operand1.desugar()?;
                new_self.operand2.desugar()?;
                new_self.as_variant()
            }
            _ => {
                debug!(span = ?self.span(), "Desugaring miscellaneous binary operator");
                Call {
                    called: Member {
                        ty: AccessType::Method,
                        name: match self.ty {
                            OprType::Add => "_add",
                            OprType::Sub => "_sub",
                            OprType::Mul => "_mul",
                            OprType::Div => "_div",
                            OprType::Mod => "_rem",
                            OprType::Eq => "_eq",
                            OprType::Ne => "_ne",
                            OprType::Lt => "_lt",
                            OprType::Le => "_le",
                            OprType::Gt => "_gt",
                            OprType::Ge => "_ge",
                            OprType::Concat => "_concat",
                            _ => unimplemented!("{:#?}", self.ty),
                        }
                        .into(),
                        name_span: None,
                        dot_span: None,
                        parent: self.operand1.desugared()?.into(),
                    }
                    .desugared()?
                    .into(),
                    paren_spans: None,
                    args: vec![self.operand2.desugared()?],
                    kwargs: HashMap::default(),
                }
                .desugared()?
            }
        })
    }

    fn interpret_expr(&self, val_symt: &mut InterpretSymTable) -> ZResult<Value> {
        let operand1 = self.operand1.interpret_expr(val_symt)?;
        let operand2 = self.operand2.interpret_expr(val_symt)?;
        match self.ty {
            OprType::And => {
                if let Value::Bool(b) = operand1 {
                    if b {
                        if let Value::Bool(b) = operand2 {
                            Ok(Value::Bool(b))
                        } else {
                            panic!()
                        }
                    } else {
                        Ok(Value::Bool(false))
                    }
                } else {
                    panic!()
                }
            }
            OprType::Or => {
                if let Value::Bool(b) = operand1 {
                    if b {
                        Ok(Value::Bool(true))
                    } else if let Value::Bool(b) = operand2 {
                        Ok(Value::Bool(b))
                    } else {
                        panic!()
                    }
                } else {
                    panic!()
                }
            }
            OprType::TypeCast => {
                operand1
                    .value_ty()
                    .namespace()
                    .get("_typecast")
                    .unwrap_or_else(|| unreachable!());
                todo!()
            }
            _opr => panic!("{_opr:?}"),
        }
    }
}

impl Reconstruct for BinaryOpr {
    fn reconstruct(&self) -> String {
        format!(
            "{} <{}> {}",
            self.operand1.reconstruct(),
            self.ty,
            self.operand2.reconstruct()
        )
    }
}
