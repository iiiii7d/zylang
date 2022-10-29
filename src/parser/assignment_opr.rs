use itertools::{Either, Itertools};
use tracing::{debug, trace};

use crate::{
    parser::buffer::{Buffer, BufferWindow},
    types::{
        element::{binary_opr::BinaryOpr, set::Set, Element, ElementVariant},
        errors::ZResult,
        position::{GetPosRaw, PosRaw},
        token::{Token, TokenType},
    },
};

impl Buffer {
    #[tracing::instrument(skip_all)]
    pub fn parse_assignment_opr(&mut self) -> ZResult<()> {
        self.reset_cursor();
        while let Some(selected) = self.next() {
            let opr_type = if let Either::Right(Token {
                ty: Some(TokenType::AssignmentOpr(opr_type)),
                ..
            }) = selected
            {
                opr_type
            } else {
                continue;
            };
            debug!(pos = ?selected.pos_raw().pos, "Parsing assignment operator");
            let var = if let Some(Either::Left(var)) = self.prev() {
                var.to_owned()
            } else {
                todo!("error")
            };
            let init_pos = var.pos_raw.pos.to_owned();
            self.next_or_err()?;
            let mut content = self.rest_incl_curr().with_as_buffer(&|buf| {
                if buf.content.is_empty() {
                    todo!("error")
                }
                buf.parse_as_expr()
            })?;
            if let Some(opr_type) = opr_type {
                debug!(?opr_type, "Desugaring");
                content = Element {
                    pos_raw: content.pos_raw.to_owned(),
                    data: Box::new(ElementVariant::BinaryOpr(BinaryOpr {
                        ty: opr_type,
                        operand1: var.to_owned(),
                        operand2: content.to_owned(),
                    })),
                };
            }
            let ele = Element {
                pos_raw: PosRaw {
                    pos: init_pos,
                    raw: self.content[self.cursor - 2..]
                        .iter()
                        .map(|a| a.pos_raw().raw)
                        .join("")
                        .into(),
                },
                data: Box::new(ElementVariant::Set(Set {
                    variable: var.to_owned(),
                    content,
                })),
            };
            trace!(?ele);
            let buffer_window = BufferWindow {
                slice: vec![Either::Left(ele)],
                range: self.cursor - 2..self.content.len(),
            };
            self.splice_buffer(buffer_window);
        }
        Ok(())
    }
}
