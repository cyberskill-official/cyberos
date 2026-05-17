use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Capability {
    pub name: String,
    pub argument: Option<String>,
}

impl Capability {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        if let Some((name, rest)) = s.split_once('(') {
            let arg = rest.trim_end_matches(')').to_owned();
            Ok(Capability { name: name.to_owned(), argument: Some(arg) })
        } else {
            Ok(Capability { name: s.to_owned(), argument: None })
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.argument {
            Some(a) => write!(f, "{}({})", self.name, a),
            None => write!(f, "{}", self.name),
        }
    }
}

#[derive(Default)]
pub struct CapabilityBroker {
    pub auto_deny: HashSet<String>,
}

impl CapabilityBroker {
    pub fn new() -> Self {
        let mut broker = Self::default();
        // Auto-deny dangerous tools until operator approval (audit §4).
        broker.auto_deny.insert("bash".to_owned());
        broker.auto_deny.insert("shell".to_owned());
        broker.auto_deny.insert("exec".to_owned());
        broker
    }

    /// Returns true if `requested` is subsumed by at least one declared cap.
    pub fn is_declared(&self, requested: &Capability, declared: &[Capability]) -> bool {
        declared.iter().any(|d| d.name == requested.name)
    }
}
