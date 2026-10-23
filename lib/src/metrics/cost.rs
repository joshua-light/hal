#[derive(Clone, Copy, Debug)]
pub struct ModelPricing {
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_read_per_mtok: f64,
    pub cache_creation_per_mtok: f64,
}

const DEFAULT_PRICING: ModelPricing = ModelPricing {
    input_per_mtok: 3.0,
    output_per_mtok: 15.0,
    cache_read_per_mtok: 0.30,
    cache_creation_per_mtok: 3.75,
};

/// Rates for `model`, falling back to Sonnet-level [`DEFAULT_PRICING`] for
/// models not in the table.
pub(super) fn pricing_for(model: &str) -> ModelPricing {
    known_pricing(model).unwrap_or(DEFAULT_PRICING)
}

/// Per-MTok rate with the 5-minute cache write at 1.25x input, which holds
/// for every Claude model on Anthropic's pricing page.
const fn rates(input: f64, output: f64, cache_read: f64) -> ModelPricing {
    ModelPricing {
        input_per_mtok: input,
        output_per_mtok: output,
        cache_read_per_mtok: cache_read,
        cache_creation_per_mtok: input * 1.25,
    }
}

/// Rates for a known Claude family, ignoring a trailing `-YYYYMMDD` suffix.
///
/// Source: https://platform.claude.com/docs/en/about-claude/pricing (checked
/// 2026-10-23). Cache reads are 0.1x input except Opus 5.5 and Sonnet 5.5
/// (0.05x) and Fable 5.1 (0.025x). Haiku 5.5 is left out: its price depends
/// on prompt length (5x above 100k tokens), which a flat rate can't express.
pub(super) fn known_pricing(model: &str) -> Option<ModelPricing> {
    match strip_date_suffix(model) {
        "claude-fable-5-1" => Some(rates(10.0, 50.0, 0.25)),
        "claude-fable-5" => Some(rates(10.0, 50.0, 1.0)),
        "claude-opus-5-5" => Some(rates(4.0, 20.0, 0.20)),
        "claude-opus-5" | "claude-opus-4-8" | "claude-opus-4-7" | "claude-opus-4-6"
        | "claude-opus-4-5" => Some(rates(5.0, 25.0, 0.50)),
        "claude-sonnet-5-5" => Some(rates(2.0, 10.0, 0.10)),
        "claude-sonnet-5" => Some(rates(2.0, 10.0, 0.20)),
        "claude-sonnet-4-7" | "claude-sonnet-4-6" | "claude-sonnet-4-5" => {
            Some(rates(3.0, 15.0, 0.30))
        }
        "claude-haiku-4-5" | "claude-haiku-4-6" => Some(rates(1.0, 5.0, 0.10)),
        _ => None,
    }
}

fn strip_date_suffix(model: &str) -> &str {
    let bytes = model.as_bytes();
    if bytes.len() >= 9 && bytes[bytes.len() - 9] == b'-' {
        let suffix = &bytes[bytes.len() - 8..];
        if suffix.iter().all(|b| b.is_ascii_digit()) {
            return &model[..model.len() - 9];
        }
    }
    model
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Tokens {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_creation: u64,
}

impl Tokens {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_creation
    }

    pub(super) fn add(&mut self, other: &Tokens) {
        self.input += other.input;
        self.output += other.output;
        self.cache_read += other.cache_read;
        self.cache_creation += other.cache_creation;
    }
}

pub(super) fn cost_of(tokens: &Tokens, p: &ModelPricing) -> f64 {
    (tokens.input as f64 * p.input_per_mtok
        + tokens.output as f64 * p.output_per_mtok
        + tokens.cache_read as f64 * p.cache_read_per_mtok
        + tokens.cache_creation as f64 * p.cache_creation_per_mtok)
        / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_model_is_priced() {
        for (label, id) in crate::agent::DEFAULT_CLAUDE_MODELS {
            assert!(known_pricing(id).is_some(), "{label} ({id}) has no pricing");
        }
    }

    fn p(id: &str) -> ModelPricing {
        known_pricing(id).unwrap_or_else(|| panic!("{id} has no pricing"))
    }

    #[test]
    fn current_claude_rates() {
        let opus = p("claude-opus-5-5");
        assert_eq!(opus.input_per_mtok, 4.0);
        assert_eq!(opus.output_per_mtok, 20.0);
        assert_eq!(opus.cache_read_per_mtok, 0.20);
        assert_eq!(opus.cache_creation_per_mtok, 5.0);
        assert_eq!(p("claude-sonnet-5-5").cache_read_per_mtok, 0.10);
        assert_eq!(p("claude-fable-5-1").cache_read_per_mtok, 0.25);
        assert_eq!(p("claude-fable-5").cache_read_per_mtok, 1.0);
        assert_eq!(p("claude-opus-4-8").output_per_mtok, 25.0);
        assert_eq!(p("claude-sonnet-5-20260801").input_per_mtok, 2.0);
        assert!(known_pricing("claude-haiku-5-5").is_none());
    }
}
