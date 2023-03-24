use std::sync::Arc;

use itertools::Itertools;

use crate::{
    ast::{Ast, AstData, Ident, Reconstruct},
    primitives::UNIT_T,
    types::position::{GetSpan, Span},
    InterpretSymTable, Type, TypecheckSymTable, Value, ZResult,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Delete {
    pub kwd_span: Option<Span>,
    pub names: Vec<Ident>,
}
impl GetSpan for Delete {
    fn span(&self) -> Option<Span> {
        self.names.merge_span(&self.kwd_span)
    }
}

impl AstData for Delete {
    fn as_variant(&self) -> Ast {
        Ast::Delete(self.to_owned())
    }

    fn typecheck(&mut self, _ty_symt: &mut TypecheckSymTable) -> ZResult<Arc<Type>> {
        Ok(Arc::clone(&UNIT_T))
    }

    fn interpret_expr(&self, val_symt: &mut InterpretSymTable) -> ZResult<Value> {
        for name in &self.names {
            val_symt.delete_val(&name.name, self)?;
        }
        Ok(Value::Unit)
    }
}

impl Reconstruct for Delete {
    fn reconstruct(&self) -> String {
        format!(
            "del {}",
            self.names.iter().map(Reconstruct::reconstruct).join(" , ")
        )
    }
}
