use std::rc::Rc;
use crate::sema::display::ProgramDisplay;
use crate::sema::values::{ActionValue, LocalReference, Value};

pub mod context;
pub mod display;
pub mod interpret;
pub mod intrinsic;
pub mod types;
pub mod values;

#[derive(Clone, Debug)]
pub struct ProgramLet {
    identifier: Rc<str>,
    parameters: Option<Box<[LocalReference]>>,
    value: Value,
}

impl ProgramLet {
    pub fn identifier(&self) -> Rc<str> {
        self.identifier.clone()
    }

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
    identifier: Rc<str>,
    kind: ProgramVariableKind,
    value: Value,
}

impl ProgramVariable {
    pub fn identifier(&self) -> Rc<str> {
        self.identifier.clone()
    }

    pub fn kind(&self) -> &ProgramVariableKind {
        &self.kind
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct ProgramAction {
    identifier: Rc<str>,
    parameters: Box<[LocalReference]>,
    action: ActionValue,
}

impl ProgramAction {
    pub fn identifier(&self) -> Rc<str> {
        self.identifier.clone()
    }

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
pub struct Program {
    lets: Box<[ProgramLet]>,
    variables: Box<[ProgramVariable]>,
    actions: Box<[ProgramAction]>,
    ticker: Option<ProgramTicker>,
    public: Option<ProgramPublic>,
    display: Option<ProgramDisplay>,
    next_local_id: u64,
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

    pub fn next_local_id(&self) -> u64 {
        self.next_local_id
    }
}
