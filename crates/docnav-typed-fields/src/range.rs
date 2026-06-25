pub type FieldLength = FieldRange<u64>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FieldNumericRange {
    #[default]
    None,
    Integer(FieldRange<i64>),
    Number(FieldRange<f64>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FieldNumericBound {
    Integer(FieldBound<i64>),
    Number(FieldBound<f64>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldBoundKind {
    Closed,
    Open,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldBound<T> {
    pub value: T,
    pub kind: FieldBoundKind,
}

impl<T> FieldBound<T> {
    pub fn closed(value: T) -> Self {
        Self {
            value,
            kind: FieldBoundKind::Closed,
        }
    }

    pub fn open(value: T) -> Self {
        Self {
            value,
            kind: FieldBoundKind::Open,
        }
    }

    pub fn is_closed(&self) -> bool {
        self.kind == FieldBoundKind::Closed
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FieldRange<T> {
    pub minimum: Option<FieldBound<T>>,
    pub maximum: Option<FieldBound<T>>,
}

impl<T> FieldRange<T> {
    pub fn min(minimum: FieldBound<T>) -> Self {
        Self {
            minimum: Some(minimum),
            maximum: None,
        }
    }

    pub fn max(maximum: FieldBound<T>) -> Self {
        Self {
            minimum: None,
            maximum: Some(maximum),
        }
    }

    pub fn between(minimum: FieldBound<T>, maximum: FieldBound<T>) -> Self {
        Self {
            minimum: Some(minimum),
            maximum: Some(maximum),
        }
    }
}
