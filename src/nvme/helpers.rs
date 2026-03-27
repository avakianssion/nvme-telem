//! Common helper functions

/// Parse ASCII field from raw c_char byte array, trimming spaces and nulls.
///
/// NVMe spec uses C char arrays for ASCII fields. This function converts
/// to unsigned bytes, then parses as UTF-8 and trims whitespace/nulls.
/// This helper function is required because serial number, model number,
/// and firmware revision are space padded.
fn parse_ascii_field(bytes: &[c_char]) -> String {
    let unsigned: Vec<u8> = bytes.iter().map(|&b| b as u8).collect();

    String::from_utf8_lossy(&unsigned)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

/// Convert [c_char; N] to [u8; N] for GUID fields.
fn convert_cchar_to_u8_array_16(bytes: &[c_char; 16]) -> [u8; 16] {
    let mut result = [0u8; 16];
    for (i, &b) in bytes.iter().enumerate() {
        result[i] = b as u8;
    }
    result
}