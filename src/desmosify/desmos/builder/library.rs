use crate::desmos::{BoxedGraphEntry, GraphBinaryKind, GraphExpression, GraphExpressionEntry};
use crate::desmos::target::DesmosTargetInfo;
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
    bool_to_internal,
    bool_from_internal,
    range_inclusive,
    range_exclusive,
    index_range_inclusive,
    index_range_exclusive,
    index_range_from,
    rectangle,
    prefix_sum,
}

impl LibraryBuilder {
    fn get_symbol(&self, subscript: impl Into<String>) -> GraphExpression {
        GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(self.prefix.clone()),
            rhs: Box::new(GraphExpression::Alphanumeric(subscript.into())),
        }
    }

    pub fn create_expression_entry(&mut self, info: &mut DesmosTargetInfo, expression: GraphExpression) -> BoxedGraphEntry {
        Box::new(GraphExpressionEntry {
            id: info.create_entry_id(),
            folder_id: self.folder_id.clone(),
            expression,
            ..Default::default()
        })
    }

    pub fn bool_to_internal(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("ToBool");

        if self.bool_to_internal.is_empty() {
            let local_restriction = info.create_local_symbol();

            // ToBool(restriction) = restrictionToBoolean(restriction)
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_restriction}])
                Equal
                ((@operatorname "restrictionToBoolean") Call [{&local_restriction}])
            );

            self.bool_to_internal = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn bool_from_internal(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("FromBool");

        if self.bool_from_internal.is_empty() {
            let local_internal = info.create_local_symbol();

            // FromBool(internal) = {restriction(internal) = 1, 0}
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_internal}])
                Equal
                (Piecewise [
                    (((@operatorname "restriction") Call [{&local_internal}]) Equal (@int 1)),
                    (@int 0),
                ])
            );

            self.bool_from_internal = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn range_inclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("RangeInc");

        if self.range_inclusive.is_empty() {
            let local_start = info.create_local_symbol();
            let local_end = info.create_local_symbol();
            let local_step = info.create_local_symbol();

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
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn range_exclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("RangeExc");

        if self.range_exclusive.is_empty() {
            let local_start = info.create_local_symbol();
            let local_end = info.create_local_symbol();
            let local_step = info.create_local_symbol();
            let local_inc = info.create_local_symbol();

            // RangeExc(start, end, step) = inc[{inc = end, 0} = 0]
            //     with inc = RangeInc(start, end, step)
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_start}, {&local_end}, {&local_step}])
                Equal
                (({&local_inc} Index ((Piecewise [
                    ({&local_inc} Equal {&local_end}), (@int 0)
                ]) Equal (@int 0))) With ({&local_inc} Equal ({self.range_inclusive(info)} Call [
                    {&local_start},
                    {&local_end},
                    {&local_step},
                ])))
            );

            self.range_exclusive = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_inclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeInc");

        if self.index_range_inclusive.is_empty() {
            let local_list = info.create_local_symbol();
            let local_start = info.create_local_symbol();
            let local_end = info.create_local_symbol();
            let local_step = info.create_local_symbol();
            let local_index = info.create_local_symbol();

            // IdxRangeInc(list, start, end, step) = [
            //     list[index] for index = RangeInc(start, end, step)
            // ]
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}, {&local_start}, {&local_end}, {&local_step}])
                Equal
                (({&local_list} Index {&local_index})
                    For ({&local_index} Equal ({self.range_inclusive(info)} Call [
                        {&local_start},
                        {&local_end},
                        {&local_step},
                    ])))
            );

            self.index_range_inclusive = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_exclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeExc");

        if self.index_range_exclusive.is_empty() {
            let local_list = info.create_local_symbol();
            let local_start = info.create_local_symbol();
            let local_end = info.create_local_symbol();
            let local_step = info.create_local_symbol();
            let local_index = info.create_local_symbol();

            // IdxRangeExc(list, start, end, step) = [
            //     list[index] for index = RangeExc(start, end, step)
            // ]
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}, {&local_start}, {&local_end}, {&local_step}])
                Equal
                (({&local_list} Index {&local_index})
                    For ({&local_index} Equal ({self.range_exclusive(info)} Call [
                        {&local_start},
                        {&local_end},
                        {&local_step},
                    ])))
            );

            self.index_range_exclusive = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_from(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeFrom");

        if self.index_range_from.is_empty() {
            let local_list = info.create_local_symbol();
            let local_start = info.create_local_symbol();
            let local_step = info.create_local_symbol();

            // IdxRangeFrom(list, start, step) = IdxRangeInc(list, start, list.count, step)
            let expression = desmos_expression!(
                ({&symbol} Call [{&local_list}, {&local_start}, {&local_step}])
                Equal
                ({self.index_range_inclusive(info)} Call [
                    {&local_list},
                    {&local_start},
                    ({&local_list} Dot (@operatorname "count")),
                    {&local_step},
                ])
            );

            self.index_range_from = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn rectangle(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("Rect");

        if self.rectangle.is_empty() {
            let local_p1 = info.create_local_symbol();
            let local_p2 = info.create_local_symbol();

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
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    /// Nevin Brackett-Rozinsky O(n) Prefix Sum (Wackscope Algorithm)
    ///
    /// https://www.desmos.com/calculator/p091kr6k84
    pub fn prefix_sum(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("PrefixSumNW");

        if self.prefix_sum.is_empty() {
            let helper_symbol = self.get_symbol("PrefixSumNWHelper");
            let local_list = info.create_local_symbol();
            let local_index = info.create_local_symbol();
            let local_wackscope_list = info.create_local_symbol();

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
                self.create_expression_entry(info, helper_expression),
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }
}
