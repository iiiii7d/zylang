use std::collections::HashMap;

use smol_str::SmolStr;
use tracing::debug;

use crate::{
    ast::{argument::Argument, Ast, AstData, Block, Declare, Ident, Reconstruct},
    types::{
        position::{GetSpan, Span},
        sym_table::FrameType,
        token::Flag,
        typeobj::TypeDefinition,
    },
    SymTable, Type, Value, ZResult,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Class {
    pub is_struct: bool,
    pub implementations: HashMap<SmolStr, Ast>,
    pub inst_fields: HashMap<SmolStr, (Ident, Option<Ast>)>,
    pub content: Option<Block>,
    pub args: Option<Vec<Argument>>,
}
impl GetSpan for Class {
    fn span(&self) -> Option<Span> {
        todo!()
    }
}

impl AstData for Class {
    fn as_variant(&self) -> Ast {
        Ast::Class(self.to_owned())
    }

    fn typecheck(&mut self, ty_symt: &mut SymTable<Type<Ast>>) -> ZResult<Type<Ast>> {
        ty_symt.add_frame(None, FrameType::Normal);
        for expr in &mut self
            .content
            .as_mut()
            .unwrap_or_else(|| unreachable!())
            .content
        {
            // TODO deal w unwrap
            expr.typecheck(ty_symt)?;
            if let Ast::Declare(Declare {
                variable,
                content,
                flags,
                ty,
                ..
            }) = &*expr
            {
                let flags = flags.iter().map(|a| a.0).collect::<Vec<_>>();
                if flags.contains(&Flag::Inst) && self.args.is_some() {
                    todo!("raise error here")
                }
                let name = if let Ast::Ident(ident) = &**variable {
                    &ident.name
                } else {
                    unimplemented!() // TODO
                };
                let ty = if let Some(ele) = ty {
                    if let Ast::Ident(ident) = &**ele {
                        ident.to_owned()
                    } else {
                        unimplemented!() // TODO
                    }
                } else {
                    todo!("infer type")
                };
                if flags.contains(&Flag::Inst) {
                    self.inst_fields
                        .insert(name.to_owned(), (ty.to_owned(), Some(*content.to_owned())));
                }
            }
        }
        if self.args.is_some() && self.implementations.contains_key("_init") {
            todo!("raise error here")
        }
        for item in self.implementations.values_mut() {
            item.typecheck(ty_symt)?;
        }
        let new_inst_fields = self
            .inst_fields
            .iter_mut()
            .map(|(ident, (ty, default))| {
                let ty = ty.typecheck(ty_symt)?;
                if let Some(default) = default {
                    if ty != default.typecheck(ty_symt)? {
                        todo!("raise error")
                    }
                }
                Ok((ident.to_owned(), (Box::new(ty), default.to_owned())))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        ty_symt.pop_frame();
        Ok(Type::Definition(TypeDefinition {
            inst_name: None,
            name: Some(if self.is_struct { "struct" } else { "class" }.into()),
            generics: vec![],
            implementations: self.implementations.to_owned(),
            inst_fields: new_inst_fields,
        }))
    }

    fn desugared(&self) -> ZResult<Ast> {
        debug!(span = ?self.span(), "Desugaring class");
        let mut new_self = self.to_owned();
        new_self.content = if let Some(content) = new_self.content {
            Some(
                content
                    .desugared()?
                    .as_block()
                    .unwrap_or_else(|| unreachable!())
                    .to_owned(),
            )
        } else {
            None
        };
        new_self
            .args
            .as_mut()
            .map(|args| {
                args.iter_mut()
                    .map(|arg| {
                        arg.desugar()?;
                        Ok(arg)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        Ok(new_self.as_variant())
    }

    fn interpret_expr(&self, val_symt: &mut SymTable<Value>) -> ZResult<Value> {
        Ok(Value::Type(Type::Definition(TypeDefinition {
            name: Some(if self.is_struct { "struct" } else { "class" }.into()),
            inst_name: None,
            generics: vec![],
            implementations: self
                .implementations
                .iter()
                .map(|(k, v)| Ok((k.to_owned(), v.interpret_expr(val_symt)?)))
                .collect::<Result<HashMap<_, _>, _>>()?,
            inst_fields: self
                .inst_fields
                .iter()
                .map(|(k, (v1, v2))| {
                    Ok((
                        k.to_owned(),
                        (
                            Box::new(if let Value::Type(value) = v1.interpret_expr(val_symt)? {
                                value
                            } else {
                                panic!()
                            }),
                            v2.to_owned()
                                .map(|v2| v2.interpret_expr(val_symt))
                                .transpose()?,
                        ),
                    ))
                })
                .collect::<Result<HashMap<_, _>, _>>()?,
        })))
    }
}

impl Reconstruct for Class {
    fn reconstruct(&self) -> String {
        format!("todo")
    }
}
