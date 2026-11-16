pub const FLOAT_LABEL: &str = "pwe";

pub fn key_passes_on(block_label: &str, program_in_front: &str, pass_keys_to: &[String]) -> bool {
    block_label == FLOAT_LABEL
        || pass_keys_to
            .iter()
            .any(|program| program == program_in_front)
}

#[cfg(test)]
mod tests;
