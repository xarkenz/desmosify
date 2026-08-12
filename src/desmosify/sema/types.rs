use crate::util::LazyConst;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::rc::Rc;
use crate::sema::values::ValueHandle;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FunctionSignature {
    pub parameter_types: Box<[TypeHandle]>,
    pub return_type: TypeHandle,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ListState {
    IsList,
    MaybeList,
}

impl ListState {
    pub fn can_coerce(from_state: Option<Self>, to_state: Option<Self>, allow_broadcast: bool) -> bool {
        from_state == to_state || match (from_state, to_state) {
            (Some(Self::IsList) | None, Some(Self::MaybeList)) => true,
            (Some(Self::IsList | Self::MaybeList), None) => allow_broadcast,
            _ => false
        }
    }

    pub fn merge(state_1: Option<Self>, state_2: Option<Self>) -> Option<Self> {
        match (state_1, state_2) {
            (Some(Self::IsList), _) => Some(Self::IsList),
            (_, Some(Self::IsList)) => Some(Self::IsList),
            (Some(Self::MaybeList), _) => Some(Self::MaybeList),
            (_, Some(Self::MaybeList)) => Some(Self::MaybeList),
            (None, None) => None,
        }
    }

    pub fn merge_all(states: impl IntoIterator<Item = Option<Self>>) -> Option<Self> {
        states.into_iter().fold(None, Self::merge)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Meta,
    Any,
    Complex,
    Real,
    Int,
    Bool,
    Color,
    Tone,
    Distribution,
    Polygon,
    Segment,
    Circle,
    Arc,
    Line,
    Ray,
    Vector,
    Angle,
    DirectedAngle,
    Segment3D,
    Triangle3D,
    Sphere3D,
    Vector3D,
    Transformation,
    InternalBool,
    Str,
    Image,
    Point2D {
        x_type: TypeHandle,
        y_type: TypeHandle,
    },
    Point3D {
        x_type: TypeHandle,
        y_type: TypeHandle,
        z_type: TypeHandle,
    },
    Enum {
        identifier: Rc<str>,
        values: Box<[(Rc<str>, ValueHandle)]>
    },
    List {
        state: ListState,
        item_type: TypeHandle,
    },
    Function {
        signature: FunctionSignature,
    },
    IntrinsicFunction,
    Action {
        parameter_types: Box<[TypeHandle]>,
    },
    Union {
        variants: Box<[TypeHandle]>,
    },
}

impl Type {
    const fn list_of(item_type: TypeHandle) -> Self {
        Self::List {
            state: ListState::IsList,
            item_type,
        }
    }

    fn union(variants: impl IntoIterator<Item = TypeHandle>) -> Self {
        Self::Union {
            variants: variants.into_iter().collect(),
        }
    }

    pub fn list_state(&self) -> Option<ListState> {
        match self {
            Self::List { state, .. } => Some(*state),
            _ => None
        }
    }

    pub fn is_first_class(&self, registry: &TypeRegistry) -> bool {
        // TODO: use this more
        match self {
            Self::Meta { .. } => false,
            Self::Any => true,
            Self::Complex => true,
            Self::Real => true,
            Self::Int => true,
            Self::Bool => true,
            Self::Color => true,
            Self::Tone => true,
            Self::Distribution => true,
            Self::Polygon => true,
            Self::Segment => true,
            Self::Circle => true,
            Self::Arc => true,
            Self::Line => true,
            Self::Ray => true,
            Self::Vector => true,
            Self::Angle => true,
            Self::DirectedAngle => true,
            Self::Segment3D => true,
            Self::Triangle3D => true,
            Self::Sphere3D => true,
            Self::Vector3D => true,
            Self::Transformation => true,
            Self::InternalBool => true,
            Self::Str => false,
            Self::Image => false,
            Self::Point2D { .. } => true,
            Self::Point3D { .. } => true,
            Self::Enum { .. } => true,
            Self::Function { .. } => false,
            Self::IntrinsicFunction => false,
            Self::Action { .. } => false,
            Self::List { .. } => true,
            Self::Union { variants } => {
                variants.iter().all(|&variant| registry.is_first_class_type(variant))
            }
        }
    }

    pub fn is_valid_var(&self, registry: &TypeRegistry) -> bool {
        // TODO: actually use this
        match *self {
            Self::Any => true,
            Self::Complex => true,
            Self::Real => true,
            Self::Int => true,
            Self::Bool => true,
            Self::Color => true,
            Self::Tone => true,
            Self::Polygon => true,
            Self::Segment => true,
            Self::Circle => true,
            Self::Arc => true,
            Self::Line => true,
            Self::Ray => true,
            Self::Vector => true,
            Self::Angle => true,
            Self::DirectedAngle => true,
            Self::Segment3D => true,
            Self::Triangle3D => true,
            Self::Sphere3D => true,
            Self::Vector3D => true,
            Self::Point2D { .. } => true,
            Self::Point3D { .. } => true,
            Self::Enum { .. } => true,
            Self::List { item_type, .. } => {
                registry.is_valid_var_type(item_type)
            }
            _ => false
        }
    }

    pub fn value_range(&self) -> Option<(Option<ValueHandle>, Option<ValueHandle>, Option<ValueHandle>)> {
        match self {
            Self::Real => Some((None, None, None)),
            Self::Int | Self::Enum { .. } => Some((
                None,
                None,
                Some(ValueHandle::ONE_INT),
            )),
            Self::Bool => Some((
                Some(ValueHandle::FALSE),
                Some(ValueHandle::TRUE),
                Some(ValueHandle::TRUE),
            )),
            _ => None
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypeHandle(NonZeroUsize);

impl TypeHandle {
    const fn new(index: usize) -> Self {
        // This may overflow if index == usize::MAX, but memory will run out before that happens.
        Self(NonZeroUsize::new(index + 1).unwrap())
    }

    const fn index(self) -> usize {
        // This is trivially guaranteed to never underflow.
        self.0.get() - 1
    }

    pub fn get(self, registry: &TypeRegistry) -> &Type {
        registry.get(self)
    }

    pub fn repr(self, registry: &TypeRegistry) -> Rc<str> {
        registry.repr(self)
    }

    pub fn into_list(self, registry: &mut TypeRegistry, state: ListState, span: Option<crate::Span>) -> crate::Result<Self> {
        registry.list_type(state, self, span)
    }

    pub fn flatten_list(self, registry: &TypeRegistry) -> (Option<ListState>, Self) {
        registry.flatten_list(self)
    }

    pub fn unflatten_list(self, registry: &mut TypeRegistry, state: Option<ListState>, span: Option<crate::Span>) -> crate::Result<Self> {
        registry.unflatten_list(state, self, span)
    }
}

impl Default for TypeHandle {
    fn default() -> Self {
        Self::ANY
    }
}

#[derive(Debug)]
pub struct TypeRegistry {
    entries: Vec<TypeEntry>,
    point_2d_handles: HashMap<[TypeHandle; 2], TypeHandle>,
    point_3d_handles: HashMap<[TypeHandle; 3], TypeHandle>,
    list_handles: HashMap<(ListState, TypeHandle), TypeHandle>,
    function_handles: HashMap<FunctionSignature, TypeHandle>,
    action_handles: HashMap<Box<[TypeHandle]>, TypeHandle>,
    union_handles: HashMap<Box<[TypeHandle]>, TypeHandle>,
}

#[derive(Clone, Debug)]
pub struct TypeEntry {
    pub definition: Type,
    pub repr: Rc<str>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            entries: Vec::new(),
            point_2d_handles: HashMap::new(),
            point_3d_handles: HashMap::new(),
            list_handles: HashMap::new(),
            function_handles: HashMap::new(),
            action_handles: HashMap::new(),
            union_handles: HashMap::new(),
        };

        for known_type in &KNOWN_TYPES {
            registry.register(known_type.get());
        }

        registry
    }

    pub fn register(&mut self, definition: Type) -> TypeHandle {
        let handle = TypeHandle::new(self.entries.len());

        match definition {
            Type::Point2D { x_type, y_type } => {
                self.point_2d_handles.insert([x_type, y_type], handle);
            }
            Type::Point3D { x_type, y_type, z_type } => {
                self.point_3d_handles.insert([x_type, y_type, z_type], handle);
            }
            Type::List { state, item_type } => {
                self.list_handles.insert((state, item_type), handle);
            }
            Type::Function { ref signature } => {
                self.function_handles.insert(signature.clone(), handle);
            }
            Type::Action { ref parameter_types } => {
                self.action_handles.insert(parameter_types.clone(), handle);
            }
            Type::Union { ref variants } => {
                self.union_handles.insert(variants.clone(), handle);
            }
            _ => {}
        }

        self.entries.push(TypeEntry {
            repr: self.compute_repr(&definition),
            definition,
        });

        handle
    }

    pub fn reregister(&mut self, handle: TypeHandle, definition: Type) {
        self.entries[handle.index()] = TypeEntry {
            repr: self.compute_repr(&definition),
            definition,
        };
    }

    pub fn entry(&self, handle: TypeHandle) -> &TypeEntry {
        &self.entries[handle.index()]
    }

    pub fn get(&self, handle: TypeHandle) -> &Type {
        &self.entries[handle.index()].definition
    }

    pub fn repr(&self, handle: TypeHandle) -> Rc<str> {
        self.entries[handle.index()].repr.clone()
    }

    pub fn point_2d_type(&mut self, x_type: TypeHandle, y_type: TypeHandle, span: Option<crate::Span>) -> crate::Result<TypeHandle> {
        if let Some(&handle) = self.point_2d_handles.get(&[x_type, y_type]) {
            return Ok(handle)
        }

        for component_type in [x_type, y_type] {
            if !self.can_coerce(component_type, TypeHandle::REAL) {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::InvalidPointComponentType {
                        component_type: self.repr(component_type),
                    },
                    span,
                }))
            }
        }

        Ok(self.register(Type::Point2D {
            x_type,
            y_type,
        }))
    }

    pub fn point_3d_type(&mut self, x_type: TypeHandle, y_type: TypeHandle, z_type: TypeHandle, span: Option<crate::Span>) -> crate::Result<TypeHandle> {
        if let Some(&handle) = self.point_3d_handles.get(&[x_type, y_type, z_type]) {
            return Ok(handle)
        }

        for component_type in [x_type, y_type, z_type] {
            if !self.can_coerce(component_type, TypeHandle::REAL) {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::InvalidPointComponentType {
                        component_type: self.repr(component_type),
                    },
                    span,
                }))
            }
        }

        Ok(self.register(Type::Point3D {
            x_type,
            y_type,
            z_type,
        }))
    }

    pub fn list_type(&mut self, state: ListState, item_type: TypeHandle, span: Option<crate::Span>) -> crate::Result<TypeHandle> {
        if let Some(&handle) = self.list_handles.get(&(state, item_type)) {
            Ok(handle)
        }
        else if self.flatten_list(item_type).0.is_some() {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidListItemType {
                    item_type: self.repr(item_type),
                },
                span,
            }))
        }
        else {
            Ok(self.register(Type::List {
                state,
                item_type,
            }))
        }
    }

    pub fn function_type(&mut self, signature: FunctionSignature) -> TypeHandle {
        if let Some(&handle) = self.function_handles.get(&signature) {
            handle
        }
        else {
            self.register(Type::Function {
                signature,
            })
        }
    }

    pub fn action_type(&mut self, parameter_types: Box<[TypeHandle]>) -> TypeHandle {
        if let Some(&handle) = self.action_handles.get(&parameter_types) {
            handle
        }
        else {
            self.register(Type::Action {
                parameter_types,
            })
        }
    }

    pub fn union_type(&mut self, variants: Box<[TypeHandle]>) -> TypeHandle {
        if let Some(&handle) = self.union_handles.get(&variants) {
            handle
        }
        else {
            self.register(Type::Union {
                variants,
            })
        }
    }

    pub fn flatten_list(&self, handle: TypeHandle) -> (Option<ListState>, TypeHandle) {
        match self.get(handle) {
            &Type::List { state, item_type } => (Some(state), item_type),
            _ => (None, handle)
        }
    }

    pub fn unflatten_list(&mut self, state: Option<ListState>, item_type: TypeHandle, span: Option<crate::Span>) -> crate::Result<TypeHandle> {
        match state {
            Some(state) => self.list_type(state, item_type, span),
            None => Ok(item_type),
        }
    }

    pub fn expect_list_type(&self, handle: TypeHandle, span: Option<crate::Span>) -> crate::Result<TypeHandle> {
        match self.get(handle) {
            // TODO: require state to be IsList?
            &Type::List { item_type, .. } => Ok(item_type),
            _ => Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedListType {
                    got_type: self.repr(handle),
                },
                span,
            }))
        }
    }

    pub fn is_first_class_type(&self, handle: TypeHandle) -> bool {
        self.get(handle).is_first_class(self)
    }

    pub fn is_valid_var_type(&self, handle: TypeHandle) -> bool {
        self.get(handle).is_valid_var(self)
    }

    pub fn coerce(&mut self, from_type: TypeHandle, to_type: TypeHandle) -> Option<TypeHandle> {
        if from_type == to_type {
            return Some(from_type)
        }

        let from_def = self.get(from_type);
        let to_def = self.get(to_type);

        if let Type::Union { variants } = to_def {
            return variants
                .clone()
                .into_iter()
                .find_map(|variant| self.coerce(from_type, variant))
        }

        match (from_def, to_def) {
            (_, Type::Any) if from_def.is_first_class(self) => Some(from_type),
            (Type::Any, _) if to_def.is_first_class(self) => Some(to_type),
            (Type::Real, Type::Complex) => Some(to_type),
            (Type::Int, Type::Real | Type::Complex) => Some(to_type),
            (Type::Bool, Type::Int | Type::Real | Type::Complex) => Some(to_type),
            (
                &Type::Point2D { x_type: from_x, y_type: from_y },
                &Type::Point2D { x_type: to_x, y_type: to_y },
            ) => {
                let x_type = self.coerce(from_x, to_x)?;
                let y_type = self.coerce(from_y, to_y)?;
                Some(self.point_2d_type(x_type, y_type, None).unwrap())
            }
            (
                &Type::Point3D { x_type: from_x, y_type: from_y, z_type: from_z },
                &Type::Point3D { x_type: to_x, y_type: to_y, z_type: to_z },
            ) => {
                let x_type = self.coerce(from_x, to_x)?;
                let y_type = self.coerce(from_y, to_y)?;
                let z_type = self.coerce(from_z, to_z)?;
                Some(self.point_3d_type(x_type, y_type, z_type, None).unwrap())
            }
            (Type::Angle, Type::Real | Type::Complex) => Some(to_type),
            (Type::DirectedAngle, Type::Real | Type::Complex) => Some(to_type),
            (Type::Enum { .. }, Type::Int | Type::Real | Type::Complex) => Some(to_type),
            _ => None
        }
    }

    pub fn can_coerce(&self, from_type: TypeHandle, to_type: TypeHandle) -> bool {
        if from_type == to_type {
            return true
        }

        let from_def = self.get(from_type);
        let to_def = self.get(to_type);

        if let Type::Union { variants } = to_def {
            return variants
                .iter()
                .any(|&variant| self.can_coerce(from_type, variant))
        }

        match (from_def, to_def) {
            (_, Type::Any) => from_def.is_first_class(self),
            (Type::Any, _) => to_def.is_first_class(self),
            (Type::Real, Type::Complex) => true,
            (Type::Int, Type::Real | Type::Complex) => true,
            (Type::Bool, Type::Int | Type::Real | Type::Complex) => true,
            (
                &Type::Point2D { x_type: from_x, y_type: from_y },
                &Type::Point2D { x_type: to_x, y_type: to_y },
            ) => self.can_coerce(from_x, to_x) && self.can_coerce(from_y, to_y),
            (
                &Type::Point3D { x_type: from_x, y_type: from_y, z_type: from_z },
                &Type::Point3D { x_type: to_x, y_type: to_y, z_type: to_z },
            ) => self.can_coerce(from_x, to_x) && self.can_coerce(from_y, to_y) && self.can_coerce(from_z, to_z),
            (Type::Angle, Type::Real | Type::Complex) => true,
            (Type::DirectedAngle, Type::Real | Type::Complex) => true,
            (Type::Enum { .. }, Type::Int | Type::Real | Type::Complex) => true,
            _ => false
        }
    }

    pub fn merge(&mut self, lhs_type: TypeHandle, rhs_type: TypeHandle, span: Option<crate::Span>) -> crate::Result<TypeHandle> {
        let (lhs_list, lhs_inner) = self.flatten_list(lhs_type);
        let (rhs_list, rhs_inner) = self.flatten_list(rhs_type);

        let merged_list = ListState::merge(lhs_list, rhs_list);
        let merged_inner = self.merge_inner(lhs_inner, rhs_inner, span)?;

        self.unflatten_list(merged_list, merged_inner, span)
    }

    pub fn merge_inner(&mut self, lhs_type: TypeHandle, rhs_type: TypeHandle, span: Option<crate::Span>) -> crate::Result<TypeHandle> {
        if lhs_type == rhs_type {
            return Ok(lhs_type)
        }

        let lhs_def = self.get(lhs_type);
        let rhs_def = self.get(rhs_type);

        match (lhs_def, rhs_def) {
            (Type::Any, _) if rhs_def.is_first_class(self) => Ok(rhs_type),
            (_, Type::Any) if lhs_def.is_first_class(self) => Ok(lhs_type),
            (
                &Type::Point2D { x_type: lhs_x, y_type: lhs_y },
                &Type::Point2D { x_type: rhs_x, y_type: rhs_y },
            ) => {
                let merged_x = self.merge_inner(lhs_x, rhs_x, span)?;
                let merged_y = self.merge_inner(lhs_y, rhs_y, span)?;
                self.point_2d_type(merged_x, merged_y, span)
            }
            (
                &Type::Point3D { x_type: lhs_x, y_type: lhs_y, z_type: lhs_z },
                &Type::Point3D { x_type: rhs_x, y_type: rhs_y, z_type: rhs_z },
            ) => {
                let merged_x = self.merge_inner(lhs_x, rhs_x, span)?;
                let merged_y = self.merge_inner(lhs_y, rhs_y, span)?;
                let merged_z = self.merge_inner(lhs_z, rhs_z, span)?;
                self.point_3d_type(merged_x, merged_y, merged_z, span)
            }
            _ => {
                for numeric_type in [
                    TypeHandle::BOOL,
                    TypeHandle::INT,
                    TypeHandle::REAL,
                    TypeHandle::COMPLEX,
                ] {
                    if self.can_coerce(lhs_type, numeric_type) && self.can_coerce(rhs_type, numeric_type) {
                        return Ok(numeric_type)
                    }
                }
                if self.can_coerce(lhs_type, rhs_type) {
                    Ok(rhs_type)
                }
                else if self.can_coerce(rhs_type, lhs_type) {
                    Ok(lhs_type)
                }
                else {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::CannotMergeTypes {
                            lhs_type: self.repr(lhs_type),
                            rhs_type: self.repr(rhs_type),
                        },
                        span,
                    }))
                }
            }
        }
    }

    pub fn expect_action_type(&self, handle: TypeHandle, min_arity: usize, argument_types: &[TypeHandle], span: Option<crate::Span>) -> crate::Result<&[TypeHandle]> {
        if let Type::Action { parameter_types } = self.get(handle) {
            if (min_arity ..= argument_types.len()).contains(&parameter_types.len())
                && std::iter::zip(argument_types, parameter_types)
                .all(|(&argument_type, &parameter_type)| {
                    self.can_coerce(argument_type, parameter_type)
                })
            {
                return Ok(parameter_types)
            }
        }
        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedActionType {
                expected_parameter_lists: (min_arity ..= argument_types.len())
                    .map(|arity| argument_types[..arity]
                        .iter()
                        .map(|&argument_type| self.repr(argument_type))
                        .collect())
                    .collect(),
                got_type: self.repr(handle),
            },
            span,
        }))
    }

    fn compute_repr(&self, definition: &Type) -> Rc<str> {
        match *definition {
            Type::Meta => "type".into(),
            Type::Any => "any".into(),
            Type::Complex => "complex".into(),
            Type::Real => "real".into(),
            Type::Int => "int".into(),
            Type::Bool => "bool".into(),
            Type::Color => "color".into(),
            Type::Tone => "tone".into(),
            Type::Distribution => "distribution".into(),
            Type::Polygon => "polygon".into(),
            Type::Segment => "segment".into(),
            Type::Circle => "circle".into(),
            Type::Arc => "arc".into(),
            Type::Line => "line".into(),
            Type::Ray => "ray".into(),
            Type::Vector => "vector".into(),
            Type::Angle => "angle".into(),
            Type::DirectedAngle => "directed_angle".into(),
            Type::Segment3D => "segment3d".into(),
            Type::Triangle3D => "triangle3d".into(),
            Type::Sphere3D => "sphere3d".into(),
            Type::Vector3D => "vector3d".into(),
            Type::Transformation => "transformation".into(),
            Type::InternalBool => "internal_bool".into(),
            Type::Str => "str".into(),
            Type::Image => "image".into(),
            Type::Point2D { x_type, y_type } => {
                format!("({}, {})", self.repr(x_type), self.repr(y_type)).into()
            }
            Type::Point3D { x_type, y_type, z_type } => {
                format!("({}, {}, {})", self.repr(x_type), self.repr(y_type), self.repr(z_type)).into()
            }
            Type::Enum { ref identifier, .. } => identifier.clone(),
            Type::Function { ref signature } => {
                let mut repr = String::from("function(");
                match *signature.parameter_types {
                    [] => {}
                    [first, ref rest @ ..] => {
                        repr.push_str(&self.repr(first));
                        for &parameter_type in rest {
                            repr.push_str(", ");
                            repr.push_str(&self.repr(parameter_type));
                        }
                    }
                }
                repr.push_str("): ");
                repr.push_str(&self.repr(signature.return_type));
                repr.into()
            }
            Type::IntrinsicFunction => "intrinsic_function".into(),
            Type::Action { ref parameter_types } => match **parameter_types {
                [] => "action()".into(),
                [first, ref rest @ ..] => {
                    let mut repr = String::from("action(");
                    repr.push_str(&self.repr(first));
                    for &parameter_type in rest {
                        repr.push_str(", ");
                        repr.push_str(&self.repr(parameter_type));
                    }
                    repr.push_str(")");
                    repr.into()
                }
            }
            Type::List { state, item_type } => match state {
                ListState::IsList => format!("[{}]", self.repr(item_type)).into(),
                ListState::MaybeList => format!("{}+", self.repr(item_type)).into(),
            }
            Type::Union { ref variants } => match **variants {
                [] => "empty_union".into(),
                [first, ref rest @ ..] => {
                    let mut repr = String::from("(");
                    repr.push_str(&self.repr(first));
                    for &variant in rest {
                        repr.push_str(" | ");
                        repr.push_str(&self.repr(variant));
                    }
                    repr.push_str(")");
                    repr.into()
                }
            }
        }
    }
}

macro_rules! known_type_handles {
    ($($(#[$meta:meta])* $handle:ident $(@ $identifier:literal)? => ($($rest:tt)+)),* $(,)?) => {
        known_type_handles!(@handle_consts 0usize, $($(#[$meta])* $handle)*);

        impl TypeHandle {
            pub fn find_primitive(identifier: &str) -> Option<Self> {
                match identifier {
                    $($($identifier => Some(Self::$handle),)?)*
                    _ => None
                }
            }
        }

        pub const KNOWN_TYPES: [LazyConst<Type>; KNOWN_TYPE_COUNT] = [
            $(known_type_handles!(@lazy_const $($rest)+),)*
        ];
    };
    // I wish I could use ${index(0)} and ${count(0)} and have it be stable.
    (@handle_consts $index:expr, $(#[$meta:meta])* $handle:ident $($rest:tt)*) => {
        impl TypeHandle {
            $(#[$meta])*
            pub const $handle: Self = Self::new($index);
        }
        known_type_handles!(@handle_consts $index + 1usize, $($rest)*);
    };
    (@handle_consts $count:expr,) => {
        const KNOWN_TYPE_COUNT: usize = $count;
    };
    // I wish I could use into() in a const context and have it be stable.
    (@lazy_const || $definition:expr) => {
        LazyConst::Deferred(|| $definition)
    };
    (@lazy_const $definition:expr) => {
        LazyConst::Immediate($definition)
    };
}

known_type_handles! {
    META => (Type::Meta),
    ANY @ "any" => (Type::Any),
    COMPLEX @ "complex" => (Type::Complex),
    REAL @ "real" => (Type::Real),
    INT @ "int" => (Type::Int),
    BOOL @ "bool" => (Type::Bool),
    COLOR @ "color" => (Type::Color),
    TONE @ "tone" => (Type::Tone),
    DISTRIBUTION @ "distribution" => (Type::Distribution),
    POLYGON @ "polygon" => (Type::Polygon),
    SEGMENT @ "segment" => (Type::Segment),
    CIRCLE @ "circle" => (Type::Circle),
    ARC @ "arc" => (Type::Arc),
    LINE @ "line" => (Type::Line),
    RAY @ "ray" => (Type::Ray),
    VECTOR @ "vector" => (Type::Vector),
    ANGLE @ "angle" => (Type::Angle),
    DIRECTED_ANGLE @ "directed_angle" => (Type::DirectedAngle),
    SEGMENT_3D @ "segment3d" => (Type::Segment3D),
    TRIANGLE_3D @ "triangle3d" => (Type::Triangle3D),
    SPHERE_3D @ "sphere3d" => (Type::Sphere3D),
    VECTOR_3D @ "vector3d" => (Type::Vector3D),
    TRANSFORMATION @ "transformation" => (Type::Transformation),
    INTERNAL_BOOL @ "internal_bool" => (Type::InternalBool),
    STR @ "str" => (Type::Str),
    IMAGE @ "image" => (Type::Image),
    INTRINSIC_FUNCTION => (Type::IntrinsicFunction),
    /// `(real, real)`
    REAL_POINT_2D => (Type::Point2D {
        x_type: TypeHandle::REAL,
        y_type: TypeHandle::REAL,
    }),
    /// `(real, real, real)`
    REAL_POINT_3D => (Type::Point3D {
        x_type: TypeHandle::REAL,
        y_type: TypeHandle::REAL,
        z_type: TypeHandle::REAL,
    }),
    /// `real | (real, real)`
    REAL_SCALAR_OR_POINT_2D => (|| Type::union([
        TypeHandle::REAL,
        TypeHandle::REAL_POINT_2D,
    ])),
    /// `real | (real, real) | (real, real, real)`
    REAL_SCALAR_OR_POINT => (|| Type::union([
        TypeHandle::REAL,
        TypeHandle::REAL_POINT_2D,
        TypeHandle::REAL_POINT_3D,
    ])),
    /// `(real, real) | (real, real, real)`
    ANY_REAL_POINT => (|| Type::union([
        TypeHandle::REAL_POINT_2D,
        TypeHandle::REAL_POINT_3D,
    ])),
    /// `segment | segment3d`
    ANY_SEGMENT => (|| Type::union([
        TypeHandle::SEGMENT,
        TypeHandle::SEGMENT_3D,
    ])),
    /// `vector | vector3d`
    ANY_VECTOR => (|| Type::union([
        TypeHandle::VECTOR,
        TypeHandle::VECTOR_3D,
    ])),
    /// `bool | int | real`
    NUMERIC_SCALAR => (|| Type::union([
        TypeHandle::BOOL,
        TypeHandle::INT,
        TypeHandle::REAL,
    ])),
    /// `int | real`
    ARITHMETIC_SCALAR => (|| Type::union([
        TypeHandle::INT,
        TypeHandle::REAL,
    ])),
    /// `(ARITHMETIC_SCALAR, ARITHMETIC_SCALAR)`
    ARITHMETIC_POINT_2D => (Type::Point2D {
        x_type: TypeHandle::ARITHMETIC_SCALAR,
        y_type: TypeHandle::ARITHMETIC_SCALAR,
    }),
    /// `(ARITHMETIC_SCALAR, ARITHMETIC_SCALAR, ARITHMETIC_SCALAR)`
    ARITHMETIC_POINT_3D => (Type::Point3D {
        x_type: TypeHandle::ARITHMETIC_SCALAR,
        y_type: TypeHandle::ARITHMETIC_SCALAR,
        z_type: TypeHandle::ARITHMETIC_SCALAR,
    }),
    /// `ARITHMETIC_SCALAR | ARITHMETIC_POINT_2D | ARITHMETIC_POINT_3D`
    ARITHMETIC_SCALAR_OR_POINT => (|| Type::union([
        TypeHandle::ARITHMETIC_SCALAR,
        TypeHandle::ARITHMETIC_POINT_2D,
        TypeHandle::ARITHMETIC_POINT_3D,
    ])),
    /// `[segment]`
    LIST_OF_SEGMENT => (Type::list_of(TypeHandle::SEGMENT)),
    /// `[angle]`
    LIST_OF_ANGLE => (Type::list_of(TypeHandle::ANGLE)),
    /// `[directed_angle]`
    LIST_OF_DIRECTED_ANGLE => (Type::list_of(TypeHandle::DIRECTED_ANGLE)),
    /// `[(real, real)]`
    LIST_OF_REAL_POINT_2D => (Type::list_of(TypeHandle::REAL_POINT_2D)),
    /// `bool | int | real | complex`
    ANY_SORTABLE => (|| Type::union([
        TypeHandle::BOOL,
        TypeHandle::INT,
        TypeHandle::REAL,
        TypeHandle::COMPLEX,
    ])),
    /// `polygon | segment | circle | arc | line | ray | vector | angle | directed_angle | (real, real)`
    ANY_TRANSFORMABLE => (|| Type::union([
        TypeHandle::POLYGON,
        TypeHandle::SEGMENT,
        TypeHandle::CIRCLE,
        TypeHandle::ARC,
        TypeHandle::LINE,
        TypeHandle::RAY,
        TypeHandle::VECTOR,
        TypeHandle::ANGLE,
        TypeHandle::DIRECTED_ANGLE,
        TypeHandle::REAL_POINT_2D,
    ])),
    /// `segment | line | ray | vector`
    ANY_LINE_LIKE => (|| Type::union([
        TypeHandle::SEGMENT,
        TypeHandle::LINE,
        TypeHandle::RAY,
        TypeHandle::VECTOR,
    ])),
    /// `segment | ray`
    SEGMENT_OR_RAY => (|| Type::union([
        TypeHandle::SEGMENT,
        TypeHandle::RAY,
    ])),
    /// `segment | circle | line | ray | arc | polygon`
    ANY_GLIDER_COMPATIBLE => (|| Type::union([
        TypeHandle::SEGMENT,
        TypeHandle::CIRCLE,
        TypeHandle::LINE,
        TypeHandle::RAY,
        TypeHandle::ARC,
        TypeHandle::POLYGON,
    ])),
}
