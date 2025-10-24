use serde::{Deserialize, Serialize};

/// Delta status state machine
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeltaStatus {
    Candidate { timestamp: String },
    Canonical { timestamp: String },
    Discarded { timestamp: String },
}

impl DeltaStatus {
    pub fn candidate(timestamp: String) -> Self {
        Self::Candidate { timestamp }
    }

    pub fn canonical(timestamp: String) -> Self {
        Self::Canonical { timestamp }
    }

    pub fn discarded(timestamp: String) -> Self {
        Self::Discarded { timestamp }
    }

    pub fn is_candidate(&self) -> bool {
        matches!(self, Self::Candidate { .. })
    }

    pub fn is_canonical(&self) -> bool {
        matches!(self, Self::Canonical { .. })
    }

    pub fn is_discarded(&self) -> bool {
        matches!(self, Self::Discarded { .. })
    }

    pub fn timestamp(&self) -> &str {
        match self {
            Self::Candidate { timestamp } => timestamp,
            Self::Canonical { timestamp } => timestamp,
            Self::Discarded { timestamp } => timestamp,
        }
    }
}

impl Default for DeltaStatus {
    fn default() -> Self {
        Self::Candidate {
            timestamp: String::new(),
        }
    }
}

/// Delta object
#[derive(Serialize, Clone, Debug, Default)]
pub struct DeltaObject {
    pub account_id: String,
    pub nonce: u64,
    pub prev_commitment: String,
    #[serde(default)]
    pub new_commitment: String,
    pub delta_payload: serde_json::Value,
    pub ack_sig: Option<String>,
    pub status: DeltaStatus,
}

impl<'de> Deserialize<'de> for DeltaObject {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeltaObjectHelper {
            account_id: String,
            nonce: u64,
            prev_commitment: String,
            #[serde(default)]
            new_commitment: String,
            delta_payload: serde_json::Value,
            ack_sig: Option<String>,
            #[serde(default)]
            status: Option<DeltaStatus>,
            #[serde(default)]
            candidate_at: Option<String>,
            #[serde(default)]
            canonical_at: Option<String>,
            #[serde(default)]
            discarded_at: Option<String>,
        }

        let helper = DeltaObjectHelper::deserialize(deserializer)?;

        let status = if let Some(status) = helper.status {
            status
        } else if let Some(discarded_at) = helper.discarded_at {
            DeltaStatus::discarded(discarded_at)
        } else if let Some(canonical_at) = helper.canonical_at {
            DeltaStatus::canonical(canonical_at)
        } else if let Some(candidate_at) = helper.candidate_at {
            DeltaStatus::candidate(candidate_at)
        } else {
            DeltaStatus::default()
        };

        Ok(DeltaObject {
            account_id: helper.account_id,
            nonce: helper.nonce,
            prev_commitment: helper.prev_commitment,
            new_commitment: helper.new_commitment,
            delta_payload: helper.delta_payload,
            ack_sig: helper.ack_sig,
            status,
        })
    }
}
