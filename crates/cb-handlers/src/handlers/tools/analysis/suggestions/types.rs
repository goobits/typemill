use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLevel {
    Safe,
    RequiresReview,
    Experimental,
}

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl FromStr for ImpactLevel {
    type Err = (); // Simple error type for when parsing fails

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(ImpactLevel::Low),
            "medium" => Ok(ImpactLevel::Medium),
            "high" => Ok(ImpactLevel::High),
            "critical" => Ok(ImpactLevel::Critical),
            // Be lenient with what the suggestion might return, similar to the old logic
            s if s.contains("low") => Ok(ImpactLevel::Low),
            s if s.contains("medium") => Ok(ImpactLevel::Medium),
            s if s.contains("high") => Ok(ImpactLevel::High),
            s if s.contains("critical") => Ok(ImpactLevel::Critical),
            _ => Err(()),
        }
    }
}
