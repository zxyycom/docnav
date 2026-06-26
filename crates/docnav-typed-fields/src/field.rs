use std::collections::BTreeMap;
use std::marker::PhantomData;

use serde_json::Value;

pub(crate) mod constraints;

use crate::extraction::{
    BuiltExtractStrategy, ExtractStrategy, ExtractionInputKind, ExtractionStrategyId,
};
use crate::metadata::{
    BuildError, DefaultMetadata, FieldConstraints, FieldIdentity, FieldPath, SchemaMetadataView,
    TypedValue, ValidationFailure, ValueKind,
};
use crate::validation::FieldValidation;
use crate::value::FieldValue;
use constraints::{
    validate_length_range, validate_numeric_range, validate_regex_pattern, value_at_path,
};

#[derive(Clone, Debug)]
pub struct FieldDef {
    pub(crate) identity: FieldIdentity,
    pub(crate) path: FieldPath,
    value_kind: ValueKind,
    constraints: FieldConstraints,
    default: DefaultMetadata,
    extractions: BTreeMap<ExtractionStrategyId, BuiltExtractStrategy>,
}

impl FieldDef {
    pub fn builder(identity: impl Into<String>) -> FieldDefBuilder {
        FieldDefBuilder::new(identity)
    }

    pub(crate) fn identity(&self) -> &FieldIdentity {
        &self.identity
    }

    pub(crate) fn schema_metadata(&self) -> SchemaMetadataView {
        self.schema_metadata_with_path(self.path.clone())
    }

    fn schema_metadata_with_path(&self, path: FieldPath) -> SchemaMetadataView {
        SchemaMetadataView {
            identity: self.identity.clone(),
            path,
            value_kind: self.value_kind,
            constraints: self.constraints.clone(),
            default: self.default.clone(),
        }
    }

    pub(crate) fn static_default_value(&self) -> Option<TypedValue> {
        self.schema_metadata()
            .static_default_value()
            .expect("static default metadata is validated during field build")
    }

    pub(crate) fn apply_declaration_presence(&mut self, required: bool) {
        self.constraints.required = required;
        self.constraints.nullable = !required;
    }

    fn validate_present_value(&self, value: &Value) -> Result<TypedValue, ValidationFailure> {
        self.schema_metadata().validate_value(value)
    }

    pub(crate) fn extraction_input_kinds(
        &self,
    ) -> impl Iterator<Item = (&ExtractionStrategyId, ExtractionInputKind)> {
        self.extractions
            .iter()
            .map(|(strategy_id, strategy)| (strategy_id, strategy.input_kind()))
    }

    pub(crate) fn decode_strategy(
        &self,
        strategy_id: &ExtractionStrategyId,
        root: &Value,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(strategy) = self.extractions.get(strategy_id) else {
            return self.schema_metadata().validate_optional_value(None);
        };
        self.validate_strategy_value(strategy, root, false)
    }

    pub(crate) fn decode_strategy_with_static_default(
        &self,
        strategy_id: &ExtractionStrategyId,
        root: &Value,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(strategy) = self.extractions.get(strategy_id) else {
            return self
                .schema_metadata()
                .validate_optional_value_with_static_default(None);
        };
        self.validate_strategy_value(strategy, root, true)
    }

    fn validate_strategy_value(
        &self,
        strategy: &BuiltExtractStrategy,
        root: &Value,
        use_static_default: bool,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(path) = strategy.json_path() else {
            return self.schema_metadata().validate_optional_value(None);
        };
        let metadata = self.schema_metadata_with_path(path.clone());
        if use_static_default {
            metadata.validate_optional_value_with_static_default(value_at_path(root, path))
        } else {
            metadata.validate_optional_value(value_at_path(root, path))
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldDefBuilder<T = ()> {
    identity: String,
    extractions: Vec<(String, ExtractStrategy)>,
    validation: Option<FieldValidation<T>>,
    default: Result<DefaultMetadata, BuildError>,
    typed: PhantomData<T>,
}

impl FieldDefBuilder<()> {
    fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            extractions: Vec::new(),
            validation: None,
            default: Ok(DefaultMetadata::None),
            typed: PhantomData,
        }
    }
}

impl<T> FieldDefBuilder<T> {
    pub fn extract(mut self, strategy_id: impl Into<String>, strategy: ExtractStrategy) -> Self {
        let strategy_id = strategy_id.into();
        if let Some((_, existing)) = self
            .extractions
            .iter_mut()
            .find(|(existing_id, _)| existing_id == &strategy_id)
        {
            *existing = strategy;
        } else {
            self.extractions.push((strategy_id, strategy));
        }
        self
    }

    pub fn validation<U>(self, validation: FieldValidation<U>) -> FieldDefBuilder<U> {
        FieldDefBuilder {
            identity: self.identity,
            extractions: self.extractions,
            validation: Some(validation),
            default: self.default,
            typed: PhantomData,
        }
    }

    pub fn default_static(mut self, value: impl Into<T>) -> Self
    where
        T: FieldValue,
    {
        let value = value.into();
        self.default = value.try_into_json_value().map(DefaultMetadata::Static);
        self
    }

    pub(crate) fn build(self) -> Result<FieldDef, BuildError> {
        let definition = self.into_definition()?;
        definition.validate_enum_metadata()?;
        definition.validate_default_metadata()?;
        Ok(definition)
    }

    fn into_definition(self) -> Result<FieldDef, BuildError> {
        let identity = FieldIdentity::new(self.identity)?;
        let extractions = build_extractions(self.extractions)?;
        let path = metadata_path(&identity, &extractions)?;
        let validation = self.validation.ok_or(BuildError::MissingValidation)?;
        let (value_kind, mut constraints) = validation.into_parts();
        constraints.nullable = !constraints.required;
        validate_numeric_range(&constraints)?;
        validate_length_range(&constraints)?;
        validate_regex_pattern(&constraints)?;
        let default = self.default?;

        Ok(FieldDef {
            identity,
            path,
            value_kind,
            constraints,
            default,
            extractions,
        })
    }
}

fn build_extractions(
    extractions: Vec<(String, ExtractStrategy)>,
) -> Result<BTreeMap<ExtractionStrategyId, BuiltExtractStrategy>, BuildError> {
    if extractions.is_empty() {
        return Err(BuildError::MissingExtractionStrategy);
    }
    let mut built = BTreeMap::new();
    for (strategy_id, strategy) in extractions {
        let strategy_id = ExtractionStrategyId::from(strategy_id)
            .validate()
            .map_err(|_| BuildError::EmptyExtractionStrategyId)?;
        let strategy = strategy.build()?;
        built.insert(strategy_id, strategy);
    }
    Ok(built)
}

fn metadata_path(
    identity: &FieldIdentity,
    extractions: &BTreeMap<ExtractionStrategyId, BuiltExtractStrategy>,
) -> Result<FieldPath, BuildError> {
    if let Some(path) = extractions
        .values()
        .find_map(BuiltExtractStrategy::json_path)
    {
        return Ok(path.clone());
    }
    FieldPath::new(identity.as_str().split('.'))
}

impl FieldDef {
    fn validate_enum_metadata(&self) -> Result<(), BuildError> {
        let Some(enum_values) = &self.constraints.enum_values else {
            return Ok(());
        };
        if enum_values.is_empty() {
            return Err(BuildError::EmptyEnumValues);
        }
        for value in enum_values {
            self.validate_present_value(value)
                .map_err(BuildError::InvalidEnumValue)?;
        }
        Ok(())
    }

    fn validate_default_metadata(&self) -> Result<(), BuildError> {
        if let DefaultMetadata::Static(value) = &self.default {
            self.validate_present_value(value)
                .map_err(BuildError::InvalidDefault)?;
        }
        Ok(())
    }
}
