use strsim::normalized_levenshtein;

/// Uses Levenshtein algorithm to get the best match of a string in a vec of strings
#[must_use]
pub(crate) fn get_best_match(data: &str, matching_set: &[String]) -> String {
    let mut best_match = &matching_set[0];
    let mut best_score = -1.0;

    for x in matching_set {
        let new_score = normalized_levenshtein(&x.to_lowercase(), &data.to_lowercase());

        if new_score > best_score {
            best_match = x;
            best_score = new_score;
        }
    }
    best_match.clone()
}

#[derive(Copy, Clone)]
pub enum StepType {
    StepUp,
    StepDown,
}

#[derive(Copy, Clone)]
pub enum DateType {
    Exact,
    Monthly,
    Yearly,
}

impl DateType {
    pub fn get_next(&mut self) -> Self {
        match self {
            DateType::Exact => DateType::Monthly,
            DateType::Monthly => DateType::Yearly,
            DateType::Yearly => DateType::Exact,
        }
    }
}
