use crate::desmos::{BoxedGraphEntry, GraphExpression, GraphExpressionEntry};
use crate::desmos::builder::fragile::FragileHandler;
use crate::desmos::target::DesmosTargetContext;
use crate::desmos_expression;

macro_rules! library_builder_definition {
    ($($intrinsic:ident),* $(,)?) => {
        pub struct LibraryBuilder {
            folder_id: Option<String>,
            prefix: GraphExpression,
            $($intrinsic: Box<[BoxedGraphEntry]>,)*
        }

        impl LibraryBuilder {
            pub fn new(folder_id: Option<String>, prefix: GraphExpression) -> Self {
                Self {
                    folder_id,
                    prefix,
                    $($intrinsic: Default::default()),*
                }
            }

            pub fn finish(self) -> impl Iterator<Item = BoxedGraphEntry> {
                [].into_iter()
                    $(.chain(self.$intrinsic))*
            }
        }
    };
}

library_builder_definition! {
    range_inclusive,
    range_exclusive,
    index_range_inclusive,
    index_range_exclusive,
    index_range_from,
    rectangle,
    compose_reducer,
    prefix_sum,
}

impl LibraryBuilder {
    fn get_symbol(&self, subscript: impl Into<String>) -> GraphExpression {
        desmos_expression!({&self.prefix} Subscript (@alnum subscript))
    }

    fn create_expression_entry(&mut self, context: &mut DesmosTargetContext, expression: GraphExpression) -> BoxedGraphEntry {
        Box::new(GraphExpressionEntry {
            id: context.create_entry_id(),
            folder_id: self.folder_id.clone(),
            expression,
            ..Default::default()
        })
    }

    pub fn range_inclusive(&mut self, context: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = self.get_symbol("RangeInc");

        if self.range_inclusive.is_empty() {
            let local_start = context.create_local_symbol();
            let local_end = context.create_local_symbol();
            let local_step = context.create_local_symbol();

            // RangeInc(start, end, step) = {
            //     start sign(step) > end sign(step): [],
            //     start + step * [0 ... floor((end - start) / step)]
            // }
            let sign_local_step = desmos_expression!(
                (@operatorname "sign") Call [{&local_step}]
            );
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_start}, {&local_end}, {&local_step}])
                Equal
                (Piecewise [
                    ((@ineq ({&local_start} ImplicitMultiply {&sign_local_step})
                        GreaterThan ({&local_end} ImplicitMultiply {&sign_local_step}))
                        Colon (List ())),
                    ({&local_start} Add ({&local_step} Multiply (List (
                        (@int 0)
                        Range
                        ((@operatorname "floor") Call [
                            (({&local_end} Subtract {&local_start}) Fraction {&local_step}),
                        ])
                    ))))
                ])
            );

            self.range_inclusive = [
                self.create_expression_entry(context, expression),
            ].into();
        }

        symbol
    }

    pub fn range_exclusive(&mut self, context: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = self.get_symbol("RangeExc");

        if self.range_exclusive.is_empty() {
            let local_start = context.create_local_symbol();
            let local_end = context.create_local_symbol();
            let local_step = context.create_local_symbol();
            let local_inc = context.create_local_symbol();

            // RangeExc(start, end, step) = inc[{inc = end, 0} = 0]
            //     with inc = RangeInc(start, end, step)
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_start}, {&local_end}, {&local_step}])
                Equal
                (({&local_inc} Index ((Piecewise [
                    ({&local_inc} Equal {&local_end}), (@int 0)
                ]) Equal (@int 0))) With ({&local_inc} Equal ({self.range_inclusive(context)} Call [
                    {&local_start},
                    {&local_end},
                    {&local_step},
                ])))
            );

            self.range_exclusive = [
                self.create_expression_entry(context, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_inclusive(&mut self, context: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeInc");

        if self.index_range_inclusive.is_empty() {
            let local_list = context.create_local_symbol();
            let local_start = context.create_local_symbol();
            let local_end = context.create_local_symbol();
            let local_step = context.create_local_symbol();
            let local_index = context.create_local_symbol();

            // IdxRangeInc(list, start, end, step) = [
            //     list[index] for index = RangeInc(start, end, step)
            // ]
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}, {&local_start}, {&local_end}, {&local_step}])
                Equal
                (({&local_list} Index {&local_index})
                    For ({&local_index} Equal ({self.range_inclusive(context)} Call [
                        {&local_start},
                        {&local_end},
                        {&local_step},
                    ])))
            );

            self.index_range_inclusive = [
                self.create_expression_entry(context, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_exclusive(&mut self, context: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeExc");

        if self.index_range_exclusive.is_empty() {
            let local_list = context.create_local_symbol();
            let local_start = context.create_local_symbol();
            let local_end = context.create_local_symbol();
            let local_step = context.create_local_symbol();
            let local_index = context.create_local_symbol();

            // IdxRangeExc(list, start, end, step) = [
            //     list[index] for index = RangeExc(start, end, step)
            // ]
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}, {&local_start}, {&local_end}, {&local_step}])
                Equal
                (({&local_list} Index {&local_index})
                    For ({&local_index} Equal ({self.range_exclusive(context)} Call [
                        {&local_start},
                        {&local_end},
                        {&local_step},
                    ])))
            );

            self.index_range_exclusive = [
                self.create_expression_entry(context, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_from(&mut self, context: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeFrom");

        if self.index_range_from.is_empty() {
            let local_list = context.create_local_symbol();
            let local_start = context.create_local_symbol();
            let local_step = context.create_local_symbol();

            // IdxRangeFrom(list, start, step) = IdxRangeInc(list, start, list.count, step)
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}, {&local_start}, {&local_step}])
                Equal
                ({self.index_range_inclusive(context)} Call [
                    {&local_list},
                    {&local_start},
                    ({&local_list} Dot (@operatorname "count")),
                    {&local_step},
                ])
            );

            self.index_range_from = [
                self.create_expression_entry(context, expression),
            ].into();
        }

        symbol
    }

    pub fn rectangle(&mut self, context: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = self.get_symbol("Rect");

        if self.rectangle.is_empty() {
            let local_p1 = context.create_local_symbol();
            let local_p2 = context.create_local_symbol();

            // Rect(p1, p2) = polygon(p1, (p2.x, p1.y), p2, (p1.x, p2.y))
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_p1}, {&local_p2}])
                Equal
                ((@operatorname "polygon") Call [
                    {&local_p1},
                    (Parentheses [
                        ({&local_p2} Dot (@letter 'x')),
                        ({&local_p1} Dot (@letter 'y')),
                    ]),
                    {&local_p2},
                    (Parentheses [
                        ({&local_p1} Dot (@letter 'x')),
                        ({&local_p2} Dot (@letter 'y')),
                    ]),
                ])
            );

            self.rectangle = [
                self.create_expression_entry(context, expression),
            ].into();
        }

        symbol
    }

    pub fn compose_reducer(&mut self, context: &mut DesmosTargetContext, fragile: &mut FragileHandler) -> GraphExpression {
        let symbol = self.get_symbol("ComposeReducer");

        if self.compose_reducer.is_empty() {
            let local_list = context.create_local_symbol();

            // ComposeReducer(list) = {
            //     list.count = 0: translation((0, 0)),
            //     list.count = 1: list[1],
            //     compose(list[1], ComposeReducer(list[2...])
            // }
            let local_list_count = desmos_expression!(
                {&local_list} Dot (@operatorname "count")
            );
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}])
                Equal
                (Piecewise [
                    (({&local_list_count} Equal (@int 0)) Colon (
                        {fragile.get_symbol("translation", 1, context)}
                        Call [(Parentheses [(@int 0), (@int 0)])]
                    )),
                    (({local_list_count} Equal (@int 1)) Colon (
                        {&local_list} Index (@int 1)
                    )),
                    ({fragile.get_symbol("compose", 2, context)} Call [
                        ({&local_list} Index (@int 1)),
                        ({&symbol} Call [({&local_list} Index ((@int 2) Range ()))]),
                    ]),
                ])
            );

            self.compose_reducer = [
                self.create_expression_entry(context, expression),
            ].into();
        }

        symbol
    }

    /// Nevin Brackett-Rozinsky O(n) Prefix Sum (Wackscope Algorithm)
    ///
    /// https://www.desmos.com/calculator/p091kr6k84
    pub fn prefix_sum(&mut self, context: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = self.get_symbol("PrefixSumNW");

        if self.prefix_sum.is_empty() {
            let helper_symbol = self.get_symbol("PrefixSumNWHelper");
            let local_list = context.create_local_symbol();
            let local_index = context.create_local_symbol();
            let local_wackscope_list = context.create_local_symbol();

            // PrefixSumNW(list) = {
            //     list.count <= 1: list,
            //     PrefixSumNWHelper([1 ... list.count]) with wackscope_list = list
            // }
            let local_list_count = desmos_expression!(
                {&local_list} Dot (@operatorname "count")
            );
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}])
                Equal
                (Piecewise [
                    ((@ineq {&local_list_count} LessThan (@int 1))
                        Colon {&local_list}),
                    (({&helper_symbol} Call [(List ((@int 1) Range {local_list_count}))])
                        With ({&local_wackscope_list} Equal {&local_list}))
                ])
            );

            // PrefixSumNWHelper(index) = {
            //     index = 1: wackscope_list[1],
            //     PrefixSumNWHelper(index - 1) + wackscope_list[index]
            // }
            let helper_expression = desmos_expression!(
                ({&helper_symbol} Call [{&local_index}])
                Equal
                (Piecewise [
                    (({&local_index} Equal (@int 1))
                        Colon ({&local_wackscope_list} Index (@int 1))),
                    (({&helper_symbol} Call [({&local_index} Subtract (@int 1))])
                        Add ({&local_wackscope_list} Index ({&local_index})))
                ])
            );

            self.prefix_sum = [
                self.create_expression_entry(context, expression),
                self.create_expression_entry(context, helper_expression),
            ].into();
        }

        symbol
    }
}
