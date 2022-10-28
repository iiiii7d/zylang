use crate::{
    types::{
        element::{ident::Ident, Element, ElementData, ElementVariant},
        position::PosRaw,
        typeobj::unit_t::UNIT_T,
    },
    InterpreterData, Print, Type, Value, ZResult,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Delete {
    pub names: Vec<Element<Ident>>,
}

impl ElementData for Delete {
    fn as_variant(&self) -> ElementVariant {
        ElementVariant::Delete(self.to_owned())
    }

    fn process<O: Print>(
        &mut self,
        _pos_raw: &PosRaw,
        _typelist: &mut InterpreterData<Type<Element>, O>,
    ) -> ZResult<Type<Element>> {
        Ok(UNIT_T.get_instance().as_type_element())
    }

    fn interpret_expr<O: Print>(&self, i_data: &mut InterpreterData<Value, O>) -> ZResult<Value> {
        for name in &self.names {
            i_data.delete_val(&name.data.name, &Default::default())?; // TODO
        }
        Ok(Value::Unit)
    }
}
