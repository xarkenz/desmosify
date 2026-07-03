use std::collections::HashMap;
use std::rc::Rc;
use crate::sema::display::ProgramDisplay;
use crate::sema::symbol::Symbol;
use crate::sema::values::{ActionValue, LocalReference, Value};

pub mod context;
pub mod display;
pub mod interpret;
pub mod intrinsic;
pub mod symbol;
pub mod types;
pub mod values;

#[derive(Clone, Debug)]
pub struct ProgramEnumeration {
    pub identifier: Rc<str>,
    pub value_identifiers: Box<[Rc<str>]>,
}

#[derive(Clone, Debug)]
pub struct ProgramImmutable {
    pub identifier: Rc<str>,
    pub parameters: Option<Box<[LocalReference]>>,
}

#[derive(Clone, Debug)]
pub enum ProgramVariableKind {
    Default,
    Timer,
    Slider {
        min: Option<Box<Value>>,
        max: Option<Box<Value>>,
        step: Option<Box<Value>>,
    },
}

#[derive(Clone, Debug)]
pub struct ProgramVariable {
    pub identifier: Rc<str>,
    pub kind: ProgramVariableKind,
}

#[derive(Clone, Debug)]
pub struct ProgramAction {
    pub identifier: Rc<str>,
    pub parameters: Box<[LocalReference]>,
    pub action: ActionValue,
}

#[derive(Clone, Debug)]
pub struct ProgramTicker {
    pub interval_ms: Option<Value>,
    pub tick_action: ActionValue,
}

#[derive(Clone, Debug)]
pub enum ProgramPublicLine {
    Expression(Value),
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
    pub symbol_values: HashMap<Symbol, Value>,
    pub enumerations: Box<[ProgramEnumeration]>,
    pub immutables: Box<[ProgramImmutable]>,
    pub variables: Box<[ProgramVariable]>,
    pub actions: Box<[ProgramAction]>,
    pub ticker: ProgramTicker,
    pub public: ProgramPublic,
    pub display: ProgramDisplay,
}
