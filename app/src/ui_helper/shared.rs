use strsim::normalized_levenshtein;

/// Uses Levenshtein algorithm to get the best match of a string in a vec of strings
#[must_use]
pub(crate) fn get_best_match(data: &str, matching_set: &[String]) -> String {
    let mut best_match = &matching_set[0];
    let mut best_score = -1.0;

    for x in matching_set {
        let new_score = normalized_levenshtein(x, data);

        if new_score > best_score {
            best_match = x;
            best_score = new_score;
        }
    }
    best_match.to_string()
}

pub enum StepType {
    StepUp,
    StepDown,
}

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


