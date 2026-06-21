use docnav_protocol::{decode_probe_result_value, ProbeResult};

use crate::adapter_process::{parse_single_json, AdapterProcessOutput};

pub fn probe_from_output(
    adapter_id: &str,
    document_path: &str,
    output: AdapterProcessOutput,
) -> Result<ProbeResult, String> {
    let value = parse_single_json(&output.stdout)?;
    let probe = decode_probe_result_value(value).map_err(|error| error.to_string())?;
    if probe.adapter_id != adapter_id {
        return Err(format!(
            "probe adapter_id {:?} does not match registry id {:?}",
            probe.adapter_id, adapter_id
        ));
    }
    if probe.path != document_path {
        return Err(format!(
            "probe path {:?} does not match requested path {:?}",
            probe.path, document_path
        ));
    }
    Ok(probe)
}
