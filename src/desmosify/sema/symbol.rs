use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Symbol {
    Global(Rc<str>),
    Action(Rc<str>),
    EnumerationValue {
        enum_identifier: Rc<str>,
        value_identifier: Rc<str>,
    },
}
