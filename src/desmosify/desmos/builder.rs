use crate::ast::RangeKind;
use crate::desmos::{BoxedGraphEntry, GraphBinaryKind, GraphExpression, GraphExpressionEntry, GraphExpressionList, GraphFolderEntry, GraphImageEntry, GraphInequalityKind, GraphSlider, GraphSliderLoopMode, GraphTextEntry, GraphTicker, GraphUnaryKind};
use crate::desmos::builder::fragile::FragileHandler;
use crate::desmos::builder::library::LibraryBuilder;
use crate::desmos::target::DesmosTargetContext;
use crate::desmos_expression;
use crate::sema::{Program, ProgramAction, ProgramImmutable, ProgramPublicEntry, ProgramPublicLine, ProgramTicker, ProgramVariable, ProgramVariableKind};
use crate::sema::display::{ImageValue, ProgramDisplayAttributeKind, ProgramDisplayElement};
use crate::sema::values::{ActionValue, ActionValueKind, BinaryKind, DoubleReducerKind, IndexKind, InequalityKind, MathematicalConstant, ParameterizedReducerKind, ReducerKind, TernaryKind, UnaryKind, ValueRegistryEntry, Value};

pub mod library;
pub mod fragile;

pub const CONSTRUCTIONS_FOLDER_ID: &str = "**dcg_geo_folder**";
pub const IMMUTABLES_FOLDER_ID: &str = "desmosify_immutables";
pub const VARIABLES_FOLDER_ID: &str = "desmosify_variables";
pub const ACTIONS_FOLDER_ID: &str = "desmosify_actions";
pub const DISPLAY_FOLDER_ID: &str = "desmosify_display";
pub const MISC_FOLDER_ID: &str = "desmosify_misc";
pub const LIBRARY_FOLDER_ID: &str = "desmosify_library";
pub const FRAGILE_FOLDER_ID: &str = "desmosify_fragile";

pub struct GraphExpressionListBuilder<'ctx> {
    context: &'ctx mut DesmosTargetContext,
    ticker: Option<GraphTicker>,
    construction_entries: Vec<BoxedGraphEntry>,
    public_entries: Vec<BoxedGraphEntry>,
    immutable_entries: Vec<BoxedGraphEntry>,
    variable_entries: Vec<BoxedGraphEntry>,
    action_entries: Vec<BoxedGraphEntry>,
    display_entries: Vec<BoxedGraphEntry>,
    misc_entries: Vec<BoxedGraphEntry>,
    library: LibraryBuilder,
    fragile: FragileHandler,
    next_dummy_noop_id: u64,
    dummy_unreachable_created: bool,
}

impl<'ctx> GraphExpressionListBuilder<'ctx> {
    pub fn build_program(program: &Program, context: &'ctx mut DesmosTargetContext) -> crate::Result<GraphExpressionList> {
        let mut builder = Self::new(context);
        builder.set_program(program)?;
        Ok(builder.finish())
    }

    pub fn new(context: &'ctx mut DesmosTargetContext) -> Self {
        Self {
            ticker: None,
            construction_entries: {
                let mut entries = Vec::<BoxedGraphEntry>::new();
                if context.descriptor().use_geometry_folder {
                    entries.push(Box::new(GraphFolderEntry {
                        id: CONSTRUCTIONS_FOLDER_ID.into(),
                        title: "geometry".into(),
                        collapsed: true,
                        secret: true,
                    }));
                }
                entries
            },
            public_entries: Vec::new(),
            immutable_entries: Vec::new(),
            variable_entries: Vec::new(),
            action_entries: Vec::new(),
            display_entries: Vec::new(),
            library: LibraryBuilder::new(
                Some(LIBRARY_FOLDER_ID.into()),
                GraphExpression::Letter('L'),
            ),
            fragile: FragileHandler::new(
                context.descriptor().fragile_strategy,
                Some(FRAGILE_FOLDER_ID),
                GraphExpression::Letter('F'),
            ),
            misc_entries: Vec::new(),
            context,
            next_dummy_noop_id: 0,
            dummy_unreachable_created: false,
        }
    }

    pub fn finish(mut self) -> GraphExpressionList {
        fn finish_folder(
            id: &str,
            title: &str,
            entries: impl IntoIterator<Item = BoxedGraphEntry>,
        ) -> impl Iterator<Item = BoxedGraphEntry> {
            let mut entries = entries.into_iter().peekable();
            entries
                .peek()
                .is_some()
                .then(|| Box::new(GraphFolderEntry {
                    id: id.into(),
                    title: title.into(),
                    collapsed: true,
                    secret: false,
                }) as BoxedGraphEntry)
                .into_iter()
                .chain(entries)
        }

        if !self.public_entries.is_empty() {
            self.public_entries.push(Box::new(GraphExpressionEntry {
                id: self.context.create_entry_id(),
                ..Default::default()
            }));
        }

        GraphExpressionList {
            ticker: self.ticker,
            entries: self.construction_entries
                .into_iter()
                .chain(self.public_entries)
                .chain(finish_folder(
                    IMMUTABLES_FOLDER_ID,
                    "desmosify: immutables",
                    self.immutable_entries,
                ))
                .chain(finish_folder(
                    VARIABLES_FOLDER_ID,
                    "desmosify: variables",
                    self.variable_entries,
                ))
                .chain(finish_folder(
                    ACTIONS_FOLDER_ID,
                    "desmosify: actions",
                    self.action_entries,
                ))
                .chain(finish_folder(
                    DISPLAY_FOLDER_ID,
                    "desmosify: display",
                    self.display_entries,
                ))
                .chain(finish_folder(
                    MISC_FOLDER_ID,
                    "desmosify: misc",
                    self.misc_entries,
                ))
                .chain(finish_folder(
                    LIBRARY_FOLDER_ID,
                    "desmosify: library",
                    self.library.finish(),
                ))
                .chain(finish_folder(
                    FRAGILE_FOLDER_ID,
                    "desmosify: fragile",
                    self.fragile.finish(),
                ))
                .collect(),
        }
    }

    pub fn create_dummy_noop(&mut self) -> GraphExpression {
        let dummy_noop_id = self.next_dummy_noop_id;
        self.next_dummy_noop_id += 1;
        let symbol = desmos_expression!(
            (@letter 'D') Subscript (@alnum format!("Noop{dummy_noop_id}"))
        );

        self.misc_entries.push(Box::new(GraphExpressionEntry {
            id: self.context.create_entry_id(),
            folder_id: Some(MISC_FOLDER_ID.into()),
            expression: desmos_expression!(
                    {&symbol} Equal (@int 0)
                ),
            ..Default::default()
        }));

        symbol
    }

    pub fn get_dummy_unreachable(&mut self) -> GraphExpression {
        let symbol = desmos_expression!(
            (@letter 'D') Subscript (@alnum "Unreachable")
        );

        if !self.dummy_unreachable_created {
            self.misc_entries.push(Box::new(GraphExpressionEntry {
                id: self.context.create_entry_id(),
                folder_id: Some(MISC_FOLDER_ID.into()),
                expression: desmos_expression!(
                    {&symbol} Equal (@int 0)
                ),
                ..Default::default()
            }));

            self.dummy_unreachable_created = true;
        }

        symbol
    }

    pub fn translate_value(&mut self, value: &ValueRegistryEntry) -> crate::Result<GraphExpression> {
        let unsupported_error = || Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedValue,
            span: value.span,
        });

        match &value.kind {
            Value::Undefined(..) => {
                // Create undefined using the alternative branch of a piecewise. This is the best
                // way to generate it reliably for any type that I can think of.
                Ok(desmos_expression!(
                    Piecewise ((@int 0) Equal (@int 1))
                ))
            }
            Value::Infinity(..) => {
                Ok(GraphExpression::Escape("infty".into()))
            }
            Value::Real(value) => {
                Ok(GraphExpression::Decimal(*value))
            }
            Value::Mathematical(kind) => {
                Ok(match kind {
                    MathematicalConstant::Pi => GraphExpression::Escape("pi".into()),
                    MathematicalConstant::Tau => GraphExpression::Escape("tau".into()),
                    MathematicalConstant::E => GraphExpression::Letter('e'),
                })
            }
            Value::Int(value) => {
                Ok(GraphExpression::Integer(*value))
            }
            Value::Bool(value) => {
                Ok(GraphExpression::Integer(*value as i64))
            }
            Value::EnumVariant { ordinal: variant_ordinal, .. } => {
                Ok(GraphExpression::Integer(*variant_ordinal))
            }
            Value::GlobalReference(reference) => {
                Ok(self.context.get_global_symbol(&reference.identifier))
            }
            Value::ActionReference(reference) => {
                Ok(self.context.get_action_symbol(&reference.identifier))
            }
            Value::Local(reference) => {
                Ok(self.context.get_local_symbol(reference.id))
            }
            Value::ViewportWidth => {
                Ok(GraphExpression::OperatorName("width".into()))
            }
            Value::ViewportHeight => {
                Ok(GraphExpression::OperatorName("height".into()))
            }
            Value::TickerDt => {
                Ok(GraphExpression::OperatorName("dt".into()))
            }
            Value::ClickIndex => {
                Ok(GraphExpression::OperatorName("index".into()))
            }
            Value::Unary { kind, operand, .. } => {
                self.translate_unary(*kind, operand, unsupported_error)
            }
            Value::Binary { kind, lhs, rhs, .. } => {
                self.translate_binary(*kind, lhs, rhs, unsupported_error)
            }
            Value::Ternary { kind, first, second, third, .. } => {
                self.translate_ternary(*kind, first, second, third, unsupported_error)
            }
            Value::InequalityChain { lhs, chain, .. } => {
                Ok(desmos_expression!(
                    Piecewise [
                        (@ineq {self.translate_value(lhs)?} [@? chain
                            .iter()
                            .map(|(kind, rhs)| Ok((
                                match kind {
                                    InequalityKind::LessThan => GraphInequalityKind::LessThan,
                                    InequalityKind::LessEqual => GraphInequalityKind::LessEqual,
                                    InequalityKind::GreaterThan => GraphInequalityKind::GreaterThan,
                                    InequalityKind::GreaterEqual => GraphInequalityKind::GreaterEqual,
                                },
                                self.translate_value(rhs)?,
                            )))]),
                        (@int 0),
                    ]
                ))
            }
            Value::Reducer { kind, list, .. } => {
                self.translate_reducer(*kind, [list.as_ref()], false, unsupported_error)
            }
            Value::ArgumentsReducer { kind, arguments, .. } => {
                self.translate_reducer(*kind, arguments, true, unsupported_error)
            }
            Value::DoubleReducer { kind, lhs_list: list_1, rhs_list: list_2, .. } => {
                self.translate_double_reducer(*kind, list_1, list_2, unsupported_error)
            }
            Value::ParameterizedReducer { kind, list, parameter, .. } => {
                self.translate_parameterized_reducer(*kind, list, parameter, unsupported_error)
            }
            Value::Join { values, .. } => {
                Ok(desmos_expression!(
                    (@operatorname "join") Call [@? values
                        .iter()
                        .map(|argument| self.translate_value(argument))]
                ))
            }
            Value::Random { source, sample_count, .. } => {
                Ok(desmos_expression!(
                    (@operatorname "random") Call [@? source.as_deref()
                        .into_iter()
                        .chain(sample_count.as_deref())
                        .map(|argument| self.translate_value(argument))]
                ))
            }
            Value::RandomSeeded { source, sample_count, seed, .. } => {
                Ok(desmos_expression!(
                    (@operatorname "random") Call [@? source.as_deref()
                        .into_iter()
                        .chain([sample_count.as_ref(), seed.as_ref()])
                        .map(|argument| self.translate_value(argument))]
                ))
            }
            Value::List { items, .. } => {
                Ok(desmos_expression!(
                    List [@? items
                        .iter()
                        .map(|item| self.translate_value(item))]
                ))
            }
            Value::ListRange { kind, start, end, step, .. } => {
                Ok(desmos_expression!(
                    {match kind {
                        RangeKind::Inclusive => self.library.range_inclusive(self.context),
                        RangeKind::Exclusive => self.library.range_exclusive(self.context),
                    }} Call [
                        {self.translate_value(start)?},
                        {self.translate_value(end)?},
                        {self.translate_value(step)?},
                    ]
                ))
            }
            Value::ListFill { value, count } => {
                Ok(desmos_expression!(
                    (@operatorname "repeat") Call [
                        {self.translate_value(value)?},
                        {self.translate_value(count)?},
                    ]
                ))
            }
            Value::ListMap { loops, value } => {
                Ok(desmos_expression!(
                    List ({self.translate_value(value)?} For [@? loops
                        .iter()
                        .rev()
                        .map(|map_loop| Ok(desmos_expression!(
                            {self.context.get_local_symbol(map_loop.local.id)}
                            Equal {self.translate_value(&map_loop.list)?}
                        )))])
                ))
            }
            Value::ListFilter { list, condition, .. } => {
                Ok(desmos_expression!(
                    {self.translate_value(list)?} Index {self.translate_condition(condition)?}
                ))
            }
            Value::Index { list, kind: operation, .. } => match operation {
                IndexKind::Single { index } => {
                    Ok(desmos_expression!(
                        {self.translate_value(list)?} Index {self.translate_value(index)?}
                    ))
                }
                IndexKind::Range { kind, from_index, to_index, step} => {
                    Ok(desmos_expression!(
                        {match kind {
                            RangeKind::Inclusive => self.library.index_range_inclusive(self.context),
                            RangeKind::Exclusive => self.library.index_range_exclusive(self.context),
                        }} Call [
                            {self.translate_value(list)?},
                            {self.translate_value(from_index)?},
                            {self.translate_value(to_index)?},
                            {self.translate_value(step)?},
                        ]
                    ))
                }
                IndexKind::RangeFrom { from_index, step } => {
                    Ok(desmos_expression!(
                        {self.library.index_range_from(self.context)} Call [
                            {self.translate_value(list)?},
                            {self.translate_value(from_index)?},
                            {self.translate_value(step)?},
                        ]
                    ))
                }
                IndexKind::RangeTo { kind, to_index } => {
                    Ok(desmos_expression!(
                        {match kind {
                            RangeKind::Inclusive => self.library.index_range_inclusive(self.context),
                            RangeKind::Exclusive => self.library.index_range_exclusive(self.context),
                        }} Call [
                            {self.translate_value(list)?},
                            (@int 1),
                            {self.translate_value(to_index)?},
                            (@int 1),
                        ]
                    ))
                }
            }
            Value::Conditional { condition_consequents, alternative, .. } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: {
                            let mut elements: Vec<_> = condition_consequents
                                .iter()
                                .map(|(condition, consequent)| {
                                    let condition = self.translate_condition(condition)?;
                                    if consequent.is_one() {
                                        Ok(condition)
                                    }
                                    else {
                                        Ok(desmos_expression!(
                                            {condition} Colon {self.translate_value(consequent)?}
                                        ))
                                    }
                                })
                                .collect::<crate::Result<_>>()?;
                            if !alternative.is_undefined() {
                                elements.push(self.translate_value(alternative)?);
                            }
                            elements
                        },
                    }),
                })
            }
            Value::UserFunctionCall { function, arguments, .. } => {
                Ok(desmos_expression!(
                    {self.translate_value(function)?} Call [@? arguments
                        .iter()
                        .map(|argument| self.translate_value(argument))]
                ))
            }
            Value::Action { parameters, action } => {
                let action = self.translate_action_value(action)?;
                let action_symbol = self.context.create_inline_action_symbol();

                let entry = Box::new(GraphExpressionEntry {
                    id: self.context.create_entry_id(),
                    folder_id: Some(ACTIONS_FOLDER_ID.into()),
                    expression: if parameters.is_empty() {
                        desmos_expression!(
                            {&action_symbol} Equal {action}
                        )
                    } else {
                        desmos_expression!(
                            ({&action_symbol} Call [@ parameters.iter().map(|parameter| {
                                self.context.get_local_symbol(parameter.id)
                            })])
                            Equal {action}
                        )
                    },
                    ..Default::default()
                });
                self.action_entries.push(entry);

                Ok(action_symbol)
            }
            _ => Err(unsupported_error())
        }
    }

    fn translate_unary(
        &mut self,
        kind: UnaryKind,
        operand: &ValueRegistryEntry,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        match kind {
            UnaryKind::Alias => {
                self.translate_value(operand)
            }
            UnaryKind::Positive => {
                Ok(desmos_expression!(
                    Parentheses (Positive {self.translate_value(operand)?})
                ))
            }
            UnaryKind::Negative => {
                Ok(desmos_expression!(
                    Parentheses (Negative {self.translate_value(operand)?})
                ))
            }
            UnaryKind::LogicalNot => {
                Ok(desmos_expression!(
                    Piecewise [
                        ({self.translate_value(operand)?} Equal (@int 0)),
                        (@int 0),
                    ]
                ))
            }
            UnaryKind::XOfPoint2D | UnaryKind::XOfPoint3D => {
                Ok(desmos_expression!(
                    {self.translate_value(operand)?} Dot (@letter 'x')
                ))
            }
            UnaryKind::YOfPoint2D | UnaryKind::YOfPoint3D => {
                Ok(desmos_expression!(
                    {self.translate_value(operand)?} Dot (@letter 'y')
                ))
            }
            UnaryKind::ZOfPoint3D => {
                Ok(desmos_expression!(
                    {self.translate_value(operand)?} Dot (@letter 'z')
                ))
            }
            UnaryKind::Sin => {
                Ok(desmos_expression!(
                    (@operatorname "sin") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Cos => {
                Ok(desmos_expression!(
                    (@operatorname "cos") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Tan => {
                Ok(desmos_expression!(
                    (@operatorname "tan") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Csc => {
                Ok(desmos_expression!(
                    (@operatorname "csc") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Sec => {
                Ok(desmos_expression!(
                    (@operatorname "sec") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Cot => {
                Ok(desmos_expression!(
                    (@operatorname "cot") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Arcsin => {
                Ok(desmos_expression!(
                    (@operatorname "arcsin") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Arccos => {
                Ok(desmos_expression!(
                    (@operatorname "arccos") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Arctan => {
                Ok(desmos_expression!(
                    (@operatorname "arctan") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Arccsc => {
                Ok(desmos_expression!(
                    (@operatorname "arccsc") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Arcsec => {
                Ok(desmos_expression!(
                    (@operatorname "arcsec") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Arccot => {
                Ok(desmos_expression!(
                    (@operatorname "arccot") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Sinh => {
                Ok(desmos_expression!(
                    (@operatorname "sinh") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Cosh => {
                Ok(desmos_expression!(
                    (@operatorname "cosh") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Tanh => {
                Ok(desmos_expression!(
                    (@operatorname "tanh") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Csch => {
                Ok(desmos_expression!(
                    (@operatorname "csch") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Sech => {
                Ok(desmos_expression!(
                    (@operatorname "sech") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Coth => {
                Ok(desmos_expression!(
                    (@operatorname "coth") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Exp => {
                Ok(desmos_expression!(
                    (@operatorname "exp") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Ln => {
                Ok(desmos_expression!(
                    (@operatorname "ln") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Ceil => {
                Ok(desmos_expression!(
                    (@operatorname "ceil") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Floor => {
                Ok(desmos_expression!(
                    (@operatorname "floor") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Round => {
                Ok(desmos_expression!(
                    (@operatorname "round") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Abs => {
                Ok(desmos_expression!(
                    Pipes {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Sign => {
                Ok(desmos_expression!(
                    (@operatorname "sign") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Sqrt => {
                Ok(GraphExpression::Radical {
                    index: None,
                    radicand: Box::new(self.translate_value(operand)?),
                })
            }
            UnaryKind::Cbrt => {
                Ok(GraphExpression::Radical {
                    index: Some(Box::new(GraphExpression::Integer(3))),
                    radicand: Box::new(self.translate_value(operand)?),
                })
            }
            UnaryKind::Factorial => {
                Ok(desmos_expression!(
                    Parentheses (Factorial {self.translate_value(operand)?})
                ))
            }
            UnaryKind::Sort => {
                Ok(desmos_expression!(
                    (@operatorname "sort") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Shuffle => {
                Ok(desmos_expression!(
                    (@operatorname "shuffle") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::Unique => {
                Ok(desmos_expression!(
                    (@operatorname "unique") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::PrefixSum => {
                Ok(desmos_expression!(
                    {self.library.prefix_sum(self.context)}
                    Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::LineFromSegment2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("lineFromSegment", 1, self.context)}
                    Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::LineFromRay2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("lineFromRay", 1, self.context)}
                    Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::AreaOfPolygon => {
                Ok(desmos_expression!(
                    (@operatorname "area") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::PerimeterOfPolygon => {
                Ok(desmos_expression!(
                    (@operatorname "perimeter") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::VerticesOfPolygon => {
                Ok(desmos_expression!(
                    (@operatorname "vertices") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::SegmentsOfPolygon => {
                Ok(desmos_expression!(
                    (@operatorname "segments") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::UndirectedAnglesOfPolygon => {
                Ok(desmos_expression!(
                    (@operatorname "angles") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::DirectedAnglesOfPolygon => {
                Ok(desmos_expression!(
                    (@operatorname "directedangles") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::RadiusOfCircle => {
                Ok(desmos_expression!(
                    (@operatorname "radius") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::CenterOfCircle => {
                Ok(desmos_expression!(
                    (@operatorname "center") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::MidpointOfSegment2D | UnaryKind::MidpointOfSegment3D => {
                Ok(desmos_expression!(
                    (@operatorname "midpoint") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::StartOfVector2D | UnaryKind::StartOfVector3D => {
                Ok(desmos_expression!(
                    (@operatorname "start") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::EndOfVector2D | UnaryKind::EndOfVector3D => {
                Ok(desmos_expression!(
                    (@operatorname "end") Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::ReflectionByLine2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("reflection", 1, self.context)}
                    Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::TranslationByPoint2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("translation", 1, self.context)}
                    Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::InverseOfTransform2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("inverse", 1, self.context)}
                    Call {self.translate_value(operand)?}
                ))
            }
            UnaryKind::BoolToInternal => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("restrictionToBoolean", 1, self.context)}
                    Call (Piecewise {self.translate_condition(operand)?})
                ))
            }
            UnaryKind::BoolFromInternal => {
                Ok(desmos_expression!(
                    Piecewise [
                        (({self.fragile.get_symbol("restriction", 1, self.context)}
                            Call [{self.translate_value(operand)?}]) Equal (@int 1)),
                        (@int 0),
                    ]
                ))
            }
        }
    }

    fn translate_binary(
        &mut self,
        kind: BinaryKind,
        lhs: &ValueRegistryEntry,
        rhs: &ValueRegistryEntry,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        match kind {
            BinaryKind::Exponent => {
                Ok(desmos_expression!(
                    Parentheses (
                        {self.translate_value(lhs)?} Superscript {self.translate_value(rhs)?}
                    )
                ))
            }
            BinaryKind::Multiply => {
                Ok(desmos_expression!(
                    Parentheses (
                        {self.translate_value(lhs)?} Multiply {self.translate_value(rhs)?}
                    )
                ))
            }
            BinaryKind::DotProduct => {
                Ok(desmos_expression!(
                    Parentheses (
                        {self.translate_value(lhs)?} DotMultiply {self.translate_value(rhs)?}
                    )
                ))
            }
            BinaryKind::CrossProduct => {
                Ok(desmos_expression!(
                    Parentheses (
                        {self.translate_value(lhs)?} CrossMultiply {self.translate_value(rhs)?}
                    )
                ))
            }
            BinaryKind::Divide => {
                Ok(desmos_expression!(
                    Parentheses (
                        {self.translate_value(lhs)?} Divide {self.translate_value(rhs)?}
                    )
                ))
            }
            BinaryKind::Remainder => {
                Ok(desmos_expression!(
                    (@operatorname "mod") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::Add => {
                Ok(desmos_expression!(
                    Parentheses (
                        {self.translate_value(lhs)?} Add {self.translate_value(rhs)?}
                    )
                ))
            }
            BinaryKind::Subtract => {
                Ok(desmos_expression!(
                    Parentheses (
                        {self.translate_value(lhs)?} Subtract {self.translate_value(rhs)?}
                    )
                ))
            }
            BinaryKind::Equal => {
                Ok(desmos_expression!(
                    Piecewise [
                        ({self.translate_value(lhs)?} Equal {self.translate_value(rhs)?}),
                        (@int 0),
                    ]
                ))
            }
            BinaryKind::NotEqual => {
                Ok(desmos_expression!(
                    Piecewise [
                        (({self.translate_value(lhs)?} Equal {self.translate_value(rhs)?})
                            Colon (@int 0)),
                        (@int 1),
                    ]
                ))
            }
            BinaryKind::LogicalAnd => {
                Ok(desmos_expression!(
                    Piecewise [
                        ({self.translate_condition(lhs)?} Colon {self.translate_value(rhs)?}),
                        (@int 0),
                    ]
                ))
            }
            BinaryKind::LogicalOr => {
                Ok(desmos_expression!(
                    Piecewise [
                        {self.translate_condition(lhs)?},
                        {self.translate_condition(rhs)?},
                        (@int 0),
                    ]
                ))
            }
            BinaryKind::Arctan2 => {
                Ok(desmos_expression!(
                    (@operatorname "arctan") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::Log => {
                Ok(desmos_expression!(
                    ((@operatorname "log") Subscript {self.translate_value(lhs)?})
                    Call {self.translate_value(rhs)?}
                ))
            }
            BinaryKind::RoundDigits => {
                Ok(desmos_expression!(
                    (@operatorname "round") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::NthRoot => {
                Ok(GraphExpression::Radical {
                    index: Some(Box::new(self.translate_value(rhs)?)),
                    radicand: Box::new(self.translate_value(lhs)?),
                })
            }
            BinaryKind::SortKeyed => {
                Ok(desmos_expression!(
                    (@operatorname "sort") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::ShuffleSeeded => {
                Ok(desmos_expression!(
                    (@operatorname "shuffle") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::Point2D => {
                Ok(desmos_expression!(
                    Parentheses [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::SegmentFromPoints2D | BinaryKind::SegmentFromPoints3D => {
                Ok(desmos_expression!(
                    (@operatorname "segment") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::LineFromPoints2D => {
                Ok(desmos_expression!(
                    (@operatorname "line") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::RayFromPoints2D => {
                Ok(desmos_expression!(
                    (@operatorname "ray") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::VectorFromPoints2D | BinaryKind::VectorFromPoints3D => {
                Ok(desmos_expression!(
                    (@operatorname "vector") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::CircleFromRadius2D | BinaryKind::CircleFromEdge2D => {
                Ok(desmos_expression!(
                    (@operatorname "circle") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::SphereFromRadius3D => {
                Ok(desmos_expression!(
                    (@operatorname "sphere") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::RectangleFromPoints2D => {
                Ok(desmos_expression!(
                    {self.library.rectangle(self.context)} Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::Glider2D => {
                Ok(desmos_expression!(
                    (@operatorname "glider") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::Reflect2D => {
                Ok(desmos_expression!(
                    (@operatorname "reflect") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::TranslateByVector2D => {
                Ok(desmos_expression!(
                    (@operatorname "translate") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::Dilation2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("dilation", 2, self.context)} Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::Rotation2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("rotation", 2, self.context)} Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::ApplyTransform2D => {
                Ok(desmos_expression!(
                    {self.fragile.get_symbol("apply", 2, self.context)} Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
            BinaryKind::MidpointOfPoints2D | BinaryKind::MidpointOfPoints3D => {
                Ok(desmos_expression!(
                    (@operatorname "midpoint") Call [
                        {self.translate_value(lhs)?},
                        {self.translate_value(rhs)?},
                    ]
                ))
            }
        }
    }

    fn translate_ternary(
        &mut self,
        kind: TernaryKind,
        first: &ValueRegistryEntry,
        second: &ValueRegistryEntry,
        third: &ValueRegistryEntry,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        let arguments = desmos_expression!([
            {self.translate_value(first)?},
            {self.translate_value(second)?},
            {self.translate_value(third)?},
        ]);

        match kind {
            TernaryKind::Point3D => {
                Ok(desmos_expression!(
                    Parentheses {arguments}
                ))
            }
            TernaryKind::Arc2D => {
                Ok(desmos_expression!(
                    (@operatorname "arc") Call {arguments}
                ))
            }
            TernaryKind::UndirectedAngle2D => {
                Ok(desmos_expression!(
                    (@operatorname "angle") Call {arguments}
                ))
            }
            TernaryKind::DirectedAngle2D => {
                Ok(desmos_expression!(
                    (@operatorname "directedangle") Call {arguments}
                ))
            }
            TernaryKind::TriangleFromVertices3D => {
                Ok(desmos_expression!(
                    (@operatorname "triangle") Call {arguments}
                ))
            }
            TernaryKind::Dilate2D => {
                Ok(desmos_expression!(
                    (@operatorname "dilate") Call {arguments}
                ))
            }
            TernaryKind::Rotate2D => {
                Ok(desmos_expression!(
                    (@operatorname "rotate") Call {arguments}
                ))
            }
            TernaryKind::TranslateByPoints2D => {
                Ok(desmos_expression!(
                    (@operatorname "translate") Call {arguments}
                ))
            }
            TernaryKind::Rgb => {
                Ok(desmos_expression!(
                    (@operatorname "rgb") Call {arguments}
                ))
            }
            TernaryKind::Hsv => {
                Ok(desmos_expression!(
                    (@operatorname "hsv") Call {arguments}
                ))
            }
            TernaryKind::Okhsv => {
                Ok(desmos_expression!(
                    (@operatorname "okhsv") Call {arguments}
                ))
            }
            TernaryKind::Oklab => {
                Ok(desmos_expression!(
                    (@operatorname "oklab") Call {arguments}
                ))
            }
            TernaryKind::Oklch => {
                Ok(desmos_expression!(
                    (@operatorname "oklch") Call {arguments}
                ))
            }
        }
    }

    fn translate_reducer<'a>(
        &mut self,
        kind: ReducerKind,
        arguments: impl IntoIterator<Item = &'a ValueRegistryEntry>,
        is_arg_reducer: bool,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        let name = match kind {
            ReducerKind::Lcm => "lcm",
            ReducerKind::Gcd => "gcd",
            ReducerKind::Mean => "mean",
            ReducerKind::Median => "median",
            ReducerKind::Min => "min",
            ReducerKind::Max => "max",
            ReducerKind::Stdev => "stdev",
            ReducerKind::Stdevp => "stdevp",
            ReducerKind::Var => "var",
            ReducerKind::Varp => "varp",
            ReducerKind::Mad => "mad",
            ReducerKind::Count => "count",
            ReducerKind::Total => "total",
            ReducerKind::PolygonFromVertices2D => "polygon",
            ReducerKind::ComposeTransforms2D => {
                // This is a fake reducer for something that is actually a binary function, so we
                // have to lower it into that format.
                return if is_arg_reducer {
                    let arguments: Vec<_> = arguments
                        .into_iter()
                        .map(|argument| self.translate_value(argument))
                        .collect::<crate::Result<_>>()?;
                    let fragile_compose = self.fragile.get_symbol("compose", 2, self.context);
                    Ok(arguments
                        .into_iter()
                        .reduce(|lhs, rhs| desmos_expression!(
                            {&fragile_compose} Call [{lhs}, {rhs}]
                        ))
                        .unwrap())
                }
                else {
                    Ok(desmos_expression!(
                        {self.library.compose_reducer(self.context, &mut self.fragile)}
                        Call {self.translate_value(arguments.into_iter().next().unwrap())?}
                    ))
                }
            }
        };

        Ok(desmos_expression!(
            (@operatorname name)
            Call [@? arguments
                .into_iter()
                .map(|argument| self.translate_value(argument))]
        ))
    }

    fn translate_double_reducer(
        &mut self,
        kind: DoubleReducerKind,
        lhs_list: &ValueRegistryEntry,
        rhs_list: &ValueRegistryEntry,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        Ok(desmos_expression!(
            (@operatorname match kind {
                DoubleReducerKind::Cov => "cov",
                DoubleReducerKind::Covp => "covp",
                DoubleReducerKind::Corr => "corr",
                DoubleReducerKind::Spearman => "spearman",
            })
            Call [
                {self.translate_value(lhs_list)?},
                {self.translate_value(rhs_list)?},
            ]
        ))
    }

    fn translate_parameterized_reducer(
        &mut self,
        kind: ParameterizedReducerKind,
        list: &ValueRegistryEntry,
        parameter: &ValueRegistryEntry,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        Ok(desmos_expression!(
            (@operatorname match kind {
                ParameterizedReducerKind::Quartile => "quartile",
                ParameterizedReducerKind::Quantile => "quantile",
                ParameterizedReducerKind::Tscore => "tscore",
            })
            Call [
                {self.translate_value(list)?},
                {self.translate_value(parameter)?},
            ]
        ))
    }

    pub fn translate_condition(&mut self, value: &ValueRegistryEntry) -> crate::Result<GraphExpression> {
        match &value.kind {
            Value::Bool(true) => {
                Ok(desmos_expression!(
                    (@int 0) Equal (@int 0)
                ))
            }
            Value::Bool(false) => {
                Ok(desmos_expression!(
                    (@int 0) Equal (@int 1)
                ))
            }
            Value::Unary { kind: UnaryKind::LogicalNot, operand, .. } => {
                Ok(desmos_expression!(
                    {self.translate_value(operand)?} Equal (@int 0)
                ))
            }
            Value::Binary { kind: BinaryKind::Equal, lhs, rhs, .. } => {
                Ok(desmos_expression!(
                    {self.translate_value(lhs)?} Equal {self.translate_value(rhs)?}
                ))
            }
            Value::Binary { kind: BinaryKind::NotEqual, lhs, rhs, .. } => {
                // If only Desmos had an operator for this... substitute with {lhs = rhs, 0} = 0
                Ok(desmos_expression!(
                    (Piecewise [
                        ({self.translate_value(lhs)?} Equal {self.translate_value(rhs)?}),
                        (@int 0),
                    ]) Equal (@int 0)
                ))
            }
            Value::InequalityChain { lhs, chain, .. } => {
                Ok(desmos_expression!(
                    (@ineq {self.translate_value(lhs)?} [@? chain
                        .iter()
                        .map(|(kind, rhs)| Ok((
                            match kind {
                                InequalityKind::LessThan => GraphInequalityKind::LessThan,
                                InequalityKind::LessEqual => GraphInequalityKind::LessEqual,
                                InequalityKind::GreaterThan => GraphInequalityKind::GreaterThan,
                                InequalityKind::GreaterEqual => GraphInequalityKind::GreaterEqual,
                            },
                            self.translate_value(rhs)?,
                        )))])
                ))
            }
            _ => {
                // Do a general "!= 0" check to evaluate a boolean: {value = 0, 0} = 0
                Ok(desmos_expression!(
                    (Piecewise [
                        ({self.translate_value(value)?} Equal (@int 0)),
                        (@int 0),
                    ]) Equal (@int 0)
                ))
            }
        }
    }

    pub fn translate_action_value(&mut self, action: &ActionValue) -> crate::Result<GraphExpression> {
        if action.is_empty() {
            // Usually omitting the action expression is not an option, so update a dummy
            // variable instead.
            return Ok(desmos_expression!(
                {self.create_dummy_noop()} RightArrow (@int 0)
            ))
        }

        match &action.kind {
            ActionValueKind::Disable => {
                // Generate an expression like {0 = 1: unreachable -> 0} since the missing
                // conditional default case is what causes the "disable" behavior.
                Ok(desmos_expression!(
                    Piecewise (((@int 0) Equal (@int 1))
                        Colon ({self.get_dummy_unreachable()} RightArrow (@int 0)))
                ))
            }
            ActionValueKind::Compound { actions } => match actions.as_ref() {
                [] => {
                    // This case was already handled above with the is_empty() check
                    unreachable!()
                }
                [action] => {
                    self.translate_action_value(action)
                }
                _ => {
                    Ok(desmos_expression!(
                        Parentheses [@? actions
                            .iter()
                            .map(|action| self.translate_action_value(action))]
                    ))
                }
            }
            ActionValueKind::Update { variable, value, .. } => {
                Ok(desmos_expression!(
                    {self.context.get_global_symbol(&variable.identifier)}
                    RightArrow {self.translate_value(value)?}
                ))
            }
            ActionValueKind::ActionCall { action, arguments, .. } => {
                if arguments.is_empty() {
                    self.translate_value(action)
                }
                else {
                    Ok(desmos_expression!(
                        {self.translate_value(action)?} Call [@? arguments
                            .iter()
                            .map(|argument| self.translate_value(argument))]
                    ))
                }
            }
            ActionValueKind::Conditional { condition_consequents, alternative } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: {
                            let mut elements: Vec<_> = condition_consequents
                                .iter()
                                .map(|(condition, consequent)| Ok(desmos_expression!(
                                    {self.translate_condition(condition)?}
                                    Colon {self.translate_action_value(consequent)?}
                                )))
                                .collect::<crate::Result<_>>()?;
                            // TODO: propagate disable
                            elements.push(self.translate_action_value(alternative)?);
                            elements
                        },
                    }),
                })
            }
        }
    }

    pub fn translate_program_immutable(&mut self, immutable: &ProgramImmutable, folder_id: Option<String>) -> crate::Result<BoxedGraphEntry> {
        let value = self.translate_value(&immutable.value)?;

        Ok(Box::new(GraphExpressionEntry {
            id: self.context.create_entry_id(),
            folder_id,
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(match &immutable.parameters {
                    Some(parameters) => GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(self.context.get_global_symbol(&immutable.identifier)),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: parameters
                                .iter()
                                .map(|parameter| self.context.get_local_symbol(parameter.id))
                                .collect(),
                        }),
                    },
                    None => self.context.get_global_symbol(&immutable.identifier),
                }),
                rhs: Box::new(value),
            },
            ..Default::default()
        }))
    }

    pub fn add_program_immutable(&mut self, immutable: &ProgramImmutable) -> crate::Result<()> {
        let entry = self.translate_program_immutable(immutable, Some(IMMUTABLES_FOLDER_ID.into()))?;
        self.immutable_entries.push(entry);
        Ok(())
    }

    pub fn translate_program_variable(&mut self, variable: &ProgramVariable, folder_id: Option<String>) -> crate::Result<BoxedGraphEntry> {
        let value = self.translate_value(&variable.value)?;

        let mut slider = match &variable.kind {
            ProgramVariableKind::Default => None,
            ProgramVariableKind::Timer => Some(GraphSlider {
                loop_mode: GraphSliderLoopMode::PlayIndefinitely,
                is_playing: true,
                ..Default::default()
            }),
            ProgramVariableKind::Slider { min, max, step } => Some(GraphSlider {
                min: min.as_ref().map_or(Ok(Default::default()), |min| {
                    self.translate_value(min)
                })?,
                max: max.as_ref().map_or(Ok(Default::default()), |max| {
                    self.translate_value(max)
                })?,
                step: step.as_ref().map_or(Ok(Default::default()), |step| {
                    self.translate_value(step)
                })?,
                ..Default::default()
            }),
        };

        if let Some((min, max, step)) = variable.value.get_type().value_range() {
            let slider = slider.get_or_insert_default();
            if let (Some(min), GraphExpression::Empty) = (&min, &slider.min) {
                slider.min = self.translate_value(min)?;
            }
            if let (Some(max), GraphExpression::Empty) = (&max, &slider.max) {
                slider.max = self.translate_value(max)?;
            }
            if let (Some(step), GraphExpression::Empty) = (&step, &slider.step) {
                slider.step = self.translate_value(step)?;
            }
        }

        Ok(Box::new(GraphExpressionEntry {
            id: self.context.create_entry_id(),
            folder_id,
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(self.context.get_global_symbol(&variable.identifier)),
                rhs: Box::new(value),
            },
            slider,
            ..Default::default()
        }))
    }

    pub fn add_program_variable(&mut self, variable: &ProgramVariable) -> crate::Result<()> {
        let entry = self.translate_program_variable(variable, Some(VARIABLES_FOLDER_ID.into()))?;
        self.variable_entries.push(entry);
        Ok(())
    }

    pub fn translate_program_action(&mut self, program_action: &ProgramAction, folder_id: Option<String>) -> crate::Result<BoxedGraphEntry> {
        let action = self.translate_action_value(&program_action.action)?;

        Ok(Box::new(GraphExpressionEntry {
            id: self.context.create_entry_id(),
            folder_id,
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(if program_action.parameters.is_empty() {
                    self.context.get_action_symbol(&program_action.identifier)
                } else {
                    GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(self.context.get_action_symbol(&program_action.identifier)),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: program_action.parameters
                                .iter()
                                .map(|parameter| self.context.get_local_symbol(parameter.id))
                                .collect(),
                        }),
                    }
                }),
                rhs: Box::new(action),
            },
            ..Default::default()
        }))
    }

    pub fn add_program_action(&mut self, program_action: &ProgramAction) -> crate::Result<()> {
        let entry = self.translate_program_action(program_action, Some(ACTIONS_FOLDER_ID.into()))?;

        self.action_entries.push(entry);

        Ok(())
    }

    pub fn set_program_ticker(&mut self, program_ticker: &ProgramTicker) -> crate::Result<()> {
        if program_ticker.tick_action.is_empty() {
            self.ticker = None;
        }
        else {
            self.ticker = Some(GraphTicker {
                playing: true,
                handler: self.translate_action_value(&program_ticker.tick_action)?,
                min_step: match &program_ticker.interval_ms {
                    Some(interval_ms) => self.translate_value(interval_ms)?,
                    None => GraphExpression::Empty,
                },
            });
        }

        Ok(())
    }

    pub fn add_public_line(&mut self, public_line: &ProgramPublicLine, folder_id: Option<String>) -> crate::Result<()> {
        let id = self.context.create_entry_id();
        let entry: BoxedGraphEntry = match public_line {
            ProgramPublicLine::Expression(value) => match &value.kind {
                Value::Str(text) => {
                    let text = text.trim();
                    if text.is_empty() {
                        Box::new(GraphExpressionEntry {
                            id,
                            folder_id,
                            ..Default::default()
                        })
                    }
                    else {
                        Box::new(GraphTextEntry {
                            id,
                            folder_id,
                            text: text.to_string(),
                        })
                    }
                }
                _ => {
                    Box::new(GraphExpressionEntry {
                        id,
                        folder_id,
                        expression: self.translate_value(value)?,
                        ..Default::default()
                    })
                }
            }
            ProgramPublicLine::Action(action) => {
                Box::new(GraphExpressionEntry {
                    id,
                    folder_id,
                    expression: self.translate_action_value(action)?,
                    ..Default::default()
                })
            }
            ProgramPublicLine::Variable(variable) => {
                self.translate_program_variable(variable, folder_id)?
            }
        };
        self.public_entries.push(entry);

        Ok(())
    }

    pub fn add_public_entry(&mut self, public_entry: &ProgramPublicEntry) -> crate::Result<()> {
        match public_entry {
            ProgramPublicEntry::Line(public_line) => {
                self.add_public_line(public_line, None)
            }
            ProgramPublicEntry::Folder { label, lines } => {
                let folder_id = self.context.create_entry_id();
                let folder_entry = Box::new(GraphFolderEntry {
                    id: folder_id.clone(),
                    title: label.to_string(),
                    collapsed: true,
                    secret: false,
                });
                self.public_entries.push(folder_entry);

                for line in lines {
                    self.add_public_line(line, Some(folder_id.clone()))?;
                }

                Ok(())
            }
        }
    }

    pub fn add_display_element(&mut self, element: &ProgramDisplayElement) -> crate::Result<()> {
        match &element.value.kind {
            Value::Image(image, _) => {
                self.add_image_display_element(element, image)
            }
            _ => {
                self.add_expression_display_element(element)
            }
        }
    }

    fn add_image_display_element(&mut self, element: &ProgramDisplayElement, image: &ImageValue) -> crate::Result<()> {
        let mut entry = GraphImageEntry {
            id: self.context.create_entry_id(),
            folder_id: Some(DISPLAY_FOLDER_ID.into()),
            image_url: image.url.to_string(),
            name: image.name.to_string(),
            background: image.background,
            center: self.translate_value(&image.center)?,
            width: self.translate_value(&image.width)?,
            height: self.translate_value(&image.height)?,
            opacity: self.translate_value(&image.opacity)?,
            angle: self.translate_value(&image.angle)?,
            ..Default::default()
        };

        for attribute in &element.attributes {
            match &attribute.kind {
                ProgramDisplayAttributeKind::Click { action } => {
                    entry.clickable.enabled = true;
                    entry.clickable.expression = self.translate_action_value(action)?;
                }
                ProgramDisplayAttributeKind::Hovered { url } => {
                    entry.clickable.hovered_image_url = url.to_string();
                }
                ProgramDisplayAttributeKind::Pressed { url } => {
                    entry.clickable.depressed_image_url = url.to_string();
                }
                ProgramDisplayAttributeKind::Description { text } => {
                    entry.clickable.description = text.to_string();
                }
                _ => panic!("given attribute is invalid for an image: {attribute:?}")
            }
        }

        self.display_entries.push(Box::new(entry));

        Ok(())
    }

    fn add_expression_display_element(&mut self, element: &ProgramDisplayElement) -> crate::Result<()> {
        let mut entry = GraphExpressionEntry {
            id: self.context.create_entry_id(),
            folder_id: Some(DISPLAY_FOLDER_ID.into()),
            expression: self.translate_value(&element.value)?,
            ..Default::default()
        };

        for attribute in &element.attributes {
            match &attribute.kind {
                ProgramDisplayAttributeKind::Color { value } => {
                    // TODO: constant => set entry.color
                    entry.color_expression = self.translate_value(value)?;
                }
                ProgramDisplayAttributeKind::Point { opacity, size, style, outline } => {
                    entry.display = true;
                    entry.point.display = true;
                    if let Some(opacity) = opacity {
                        entry.point.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(size) = size {
                        entry.point.size = self.translate_value(size)?;
                    }
                    entry.point.style = *style;
                    entry.point.outline = *outline;
                }
                ProgramDisplayAttributeKind::Drag { mode } => {
                    entry.point.drag_mode = *mode;
                }
                ProgramDisplayAttributeKind::Label { text, opacity, size, angle, orientation, outline } => {
                    // Don't set entry.display = true, that will show points as well
                    entry.label.display = true;
                    entry.label.text = text.to_string();
                    if let Some(opacity) = opacity {
                        entry.label.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(size) = size {
                        entry.label.size = self.translate_value(size)?;
                    }
                    if let Some(angle) = angle {
                        entry.label.angle = self.translate_value(angle)?;
                    }
                    entry.label.orientation = *orientation;
                    entry.label.outline = *outline;
                }
                ProgramDisplayAttributeKind::Line { opacity, width, style } => {
                    entry.display = true;
                    entry.line.display = true;
                    if let Some(opacity) = opacity {
                        entry.line.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(width) = width {
                        entry.line.width = self.translate_value(width)?;
                    }
                    entry.line.style = *style;
                }
                ProgramDisplayAttributeKind::Fill { opacity } => {
                    entry.display = true;
                    entry.fill.display = true;
                    if let Some(opacity) = opacity {
                        entry.fill.opacity = self.translate_value(opacity)?;
                    }
                }
                ProgramDisplayAttributeKind::Click { action } => {
                    entry.clickable.enabled = true;
                    entry.clickable.expression = self.translate_action_value(action)?;
                }
                ProgramDisplayAttributeKind::Description { text } => {
                    entry.clickable.description = text.to_string();
                }
                _ => panic!("given attribute is invalid for an expression: {attribute:?}")
            }
        }

        self.display_entries.push(Box::new(entry));

        Ok(())
    }

    pub fn set_program(&mut self, program: &Program) -> crate::Result<()> {
        for immutable in &program.immutables {
            self.add_program_immutable(immutable)?;
        }
        for variable in &program.variables {
            self.add_program_variable(variable)?;
        }
        for action in &program.actions {
            self.add_program_action(action)?;
        }
        self.set_program_ticker(&program.tickers)?;
        for public_entry in &program.public_lists.entries {
            self.add_public_entry(public_entry)?;
        }
        for display_element in &program.display_lists.elements {
            self.add_display_element(display_element)?;
        }

        Ok(())
    }
}
