use docnav_protocol::{decode_manifest_value, Manifest, Operation};

use crate::adapter_process::{parse_single_json, AdapterProcessOutput};

pub fn manifest_from_output(
    adapter_id: &str,
    output: AdapterProcessOutput,
) -> Result<Manifest, String> {
    let value = parse_single_json(&output.stdout)?;
    let manifest = decode_manifest_value(value).map_err(|error| error.to_string())?;
    if manifest.adapter.id != adapter_id {
        return Err(format!(
            "manifest adapter id {:?} does not match registry id {:?}",
            manifest.adapter.id, adapter_id
        ));
    }
    Ok(manifest)
}

pub fn ensure_capability(manifest: &Manifest, operation: Operation) -> Result<(), String> {
    if manifest.capabilities.contains(&operation) {
        Ok(())
    } else {
        Err(format!("adapter does not declare capability {operation}"))
    }
}
