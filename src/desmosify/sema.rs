use std::rc::Rc;
use crate::sema::values::{ActionValue, LocalReference, Value};

pub mod context;
pub mod interpret;
pub mod intrinsic;
pub mod types;
pub mod values;

#[derive(Clone, Debug)]
pub struct ProgramLet {
    parameters: Option<Box<[LocalReference]>>,
    value: Value,
}

impl ProgramLet {
    pub fn parameters(&self) -> Option<&[LocalReference]> {
        self.parameters.as_deref()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub enum ProgramVariableKind {
    Default,
    Timer,
}

#[derive(Clone, Debug)]
pub struct ProgramVariable {
    kind: ProgramVariableKind,
    value: Value,
}

impl ProgramVariable {
    pub fn kind(&self) -> &ProgramVariableKind {
        &self.kind
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct ProgramAction {
    parameters: Box<[LocalReference]>,
    action: ActionValue,
}

impl ProgramAction {
    pub fn parameters(&self) -> &[LocalReference] {
        &self.parameters
    }

    pub fn action(&self) -> &ActionValue {
        &self.action
    }
}

#[derive(Clone, Debug)]
pub struct ProgramTicker {
    interval_ms: Option<Value>,
    tick_action: ActionValue,
}

impl ProgramTicker {
    pub fn interval_ms(&self) -> Option<&Value> {
        self.interval_ms.as_ref()
    }

    pub fn tick_action(&self) -> &ActionValue {
        &self.tick_action
    }
}

#[derive(Clone, Debug)]
pub enum ProgramPublicLine {
    Text(Rc<str>),
    Expression(Value),
    Action(ActionValue),
}

#[derive(Clone, Debug)]
pub struct ProgramPublic {
    lines: Box<[ProgramPublicLine]>,
}

impl ProgramPublic {
    pub fn lines(&self) -> &[ProgramPublicLine] {
        &self.lines
    }
}

#[derive(Clone, Debug)]
pub enum ProgramDisplayAttributeValue {
    Arguments(Box<[Value]>),
    Action(ActionValue),
}

#[derive(Clone, Debug)]
pub struct ProgramDisplayAttribute {
    key: Rc<str>,
    value: ProgramDisplayAttributeValue,
}

impl ProgramDisplayAttribute {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &ProgramDisplayAttributeValue {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct ProgramDisplayElement {
    expression: Value,
    attributes: Box<[ProgramDisplayAttribute]>,
}

impl ProgramDisplayElement {
    pub fn expression(&self) -> &Value {
        &self.expression
    }

    pub fn attributes(&self) -> &[ProgramDisplayAttribute] {
        &self.attributes
    }
}

#[derive(Clone, Debug)]
pub struct ProgramDisplay {
    elements: Box<[ProgramDisplayElement]>,
}

impl ProgramDisplay {
    pub fn elements(&self) -> &[ProgramDisplayElement] {
        &self.elements
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    lets: Box<[ProgramLet]>,
    variables: Box<[ProgramVariable]>,
    actions: Box<[ProgramAction]>,
    ticker: Option<ProgramTicker>,
    public: Option<ProgramPublic>,
    display: Option<ProgramDisplay>,
}

impl Program {
    pub fn lets(&self) -> &[ProgramLet] {
        &self.lets
    }

    pub fn variables(&self) -> &[ProgramVariable] {
        &self.variables
    }

    pub fn actions(&self) -> &[ProgramAction] {
        &self.actions
    }

    pub fn ticker(&self) -> Option<&ProgramTicker> {
        self.ticker.as_ref()
    }

    pub fn public(&self) -> Option<&ProgramPublic> {
        self.public.as_ref()
    }

    pub fn display(&self) -> Option<&ProgramDisplay> {
        self.display.as_ref()
    }
}
