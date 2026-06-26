extern crate self as docnav_typed_fields;

mod extraction;
mod field;
mod metadata;
mod range;
mod set;
mod validation;
mod value;

pub use docnav_typed_fields_macros::FieldDefs;
pub use extraction::{
    ExtractStrategy, ExtractionInputKind, ExtractionStrategyId, InvalidExtractionStrategyId,
};
pub use field::{FieldDef, FieldDefBuilder};
pub use metadata::{
    ActualValueKind, BuildError, DefaultMetadata, FieldConstraints, FieldDuplicateIdentityError,
    FieldIdentity, FieldPath, SchemaMetadataView, StrategyMetadataView, TypedValue,
    ValidationFailure, ValidationReason, ValueKind,
};
pub use range::{
    FieldBound, FieldBoundKind, FieldLength, FieldNumericBound, FieldNumericRange, FieldRange,
};
pub use serde_json::Value as JsonValue;
pub use set::{
    ExpectedFieldShape, FieldDefBuildFailure, FieldDefSet, FieldDefSetBuildError,
    FieldExtractionError, FieldValidationErrors,
};
pub use validation::FieldValidation;
pub use value::{FieldStringEnum, FieldValue, FieldValueError};

pub trait FieldDefs: Sized {
    type DefinitionSet;
    type Builder: FieldDefsBuilder<DefinitionSet = Self::DefinitionSet>;
    type DefaultValues;

    #[doc(hidden)]
    const __FIELD_COUNT: usize;

    fn field_defs_builder() -> Self::Builder;

    fn field_defs() -> Result<Self::DefinitionSet, FieldDefSetBuildError> {
        Self::field_defs_builder().build()
    }

    #[doc(hidden)]
    fn __values_from_slots(values: &__private::FieldValues, offset: usize) -> Self;

    #[doc(hidden)]
    fn __default_values_from_slots(
        values: &__private::FieldValues,
        offset: usize,
    ) -> Self::DefaultValues;
}

pub trait FieldDefsBuilder: Sized + Clone {
    type DefinitionSet;

    fn build(self) -> Result<Self::DefinitionSet, FieldDefSetBuildError>;

    #[doc(hidden)]
    fn __register_field_defs(
        self,
        builder: __private::FieldDefSetBuilder,
        declaration_path: Vec<String>,
    ) -> __private::FieldDefSetBuilder;
}

#[doc(hidden)]
pub mod __private {
    pub use crate::set::{ExpectedFieldShape, FieldDefSetBuilder, FieldValues};
    pub use crate::JsonValue;
}

#[cfg(test)]
mod tests;
