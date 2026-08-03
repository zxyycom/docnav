use crate::routing::{
    select_adapter, AdapterSelection, AdapterSelectionRequest, NavigationAdapterRegistry,
};
use crate::NavigationError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationContextSelection {
    pub adapter_id: String,
    pub source: String,
    pub note: String,
}

impl NavigationContextSelection {
    pub(crate) fn from_selection(
        selection: &AdapterSelection<'_>,
        preselected_adapter_id: Option<&str>,
        preselected_adapter_source: &str,
    ) -> Self {
        let source = if preselected_adapter_id.is_some() {
            preselected_adapter_source
        } else {
            "automatic_discovery"
        };
        Self {
            adapter_id: selection.adapter.id().to_owned(),
            source: source.to_owned(),
            note: "selected built-in adapter resolved from static registry".to_owned(),
        }
    }
}

pub fn select_navigation_context<R>(
    registry: &R,
    document_path: &str,
    preselected_adapter_id: Option<&str>,
    preselected_adapter_source: &str,
) -> Result<NavigationContextSelection, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let selection = select_adapter(AdapterSelectionRequest {
        registry,
        document_path,
        preselected_adapter_id,
        preselected_adapter_source,
    })?;
    Ok(NavigationContextSelection::from_selection(
        &selection,
        preselected_adapter_id,
        preselected_adapter_source,
    ))
}
