use std::rc::Rc;
use crate::sema::display::ProgramDisplay;
use crate::sema::types::TypeHandle;
use crate::sema::values::{ActionValue, ValueHandle};

pub mod context;
pub mod display;
pub mod interpret;
pub mod intrinsic;
pub mod types;
pub mod values;

#[derive(Clone, Debug)]
pub struct ProgramEnumeration {
    pub identifier: Rc<str>,
    pub type_handle: TypeHandle,
}

#[derive(Clone, Debug)]
pub struct ProgramImmutable {
    pub identifier: Rc<str>,
    pub parameters: Option<Box<[ValueHandle]>>,
    pub value: ValueHandle,
}

#[derive(Clone, Debug)]
pub enum ProgramVariableKind {
    Default,
    Timer,
    Slider {
        min: Option<ValueHandle>,
        max: Option<ValueHandle>,
        step: Option<ValueHandle>,
    },
}

#[derive(Clone, Debug)]
pub struct ProgramVariable {
    pub identifier: Rc<str>,
    pub kind: ProgramVariableKind,
    pub value: ValueHandle,
}

#[derive(Clone, Debug)]
pub struct ProgramAction {
    pub identifier: Rc<str>,
    pub parameters: Box<[ValueHandle]>,
    pub action: ActionValue,
}

#[derive(Clone, Debug)]
pub struct ProgramTicker {
    pub interval_ms: Option<ValueHandle>,
    pub tick_action: ActionValue,
}

#[derive(Clone, Debug)]
pub enum ProgramPublicLine {
    Expression(ValueHandle),
    Action(ActionValue),
    Variable(ProgramVariable),
}

#[derive(Clone, Debug)]
pub enum ProgramPublicEntry {
    Line(ProgramPublicLine),
    Folder {
        label: Rc<str>,
        lines: Box<[ProgramPublicLine]>,
    },
}

#[derive(Clone, Debug)]
pub struct ProgramPublic {
    pub entries: Box<[ProgramPublicEntry]>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub enumerations: Box<[ProgramEnumeration]>,
    pub immutables: Box<[ProgramImmutable]>,
    pub variables: Box<[ProgramVariable]>,
    pub actions: Box<[ProgramAction]>,
    pub ticker: ProgramTicker,
    pub public: ProgramPublic,
    pub display: ProgramDisplay,
}
