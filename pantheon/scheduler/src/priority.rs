use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PriorityLevel {
    Background = 1,
    Low = 2,
    Normal = 3,
    High = 4,
    Critical = 5,
}

impl PriorityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PriorityLevel::Background => "background",
            PriorityLevel::Low => "low",
            PriorityLevel::Normal => "normal",
            PriorityLevel::High => "high",
            PriorityLevel::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "background" => Some(PriorityLevel::Background),
            "low" => Some(PriorityLevel::Low),
            "normal" => Some(PriorityLevel::Normal),
            "high" => Some(PriorityLevel::High),
            "critical" => Some(PriorityLevel::Critical),
            _ => None,
        }
    }

    pub fn weight(&self) -> u64 {
        *self as u64
    }
}

impl fmt::Display for PriorityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingPolicy {
    Fifo,
    Priority,
    RoundRobin,
    Deadline,
}

impl SchedulingPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulingPolicy::Fifo => "fifo",
            SchedulingPolicy::Priority => "priority",
            SchedulingPolicy::RoundRobin => "round-robin",
            SchedulingPolicy::Deadline => "deadline",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fifo" => Some(SchedulingPolicy::Fifo),
            "priority" => Some(SchedulingPolicy::Priority),
            "round-robin" | "round_robin" => Some(SchedulingPolicy::RoundRobin),
            "deadline" => Some(SchedulingPolicy::Deadline),
            _ => None,
        }
    }
}

impl fmt::Display for SchedulingPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for SchedulingPolicy {
    fn default() -> Self {
        SchedulingPolicy::Priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_level_ordering() {
        assert!(PriorityLevel::Critical > PriorityLevel::High);
        assert!(PriorityLevel::High > PriorityLevel::Normal);
        assert!(PriorityLevel::Normal > PriorityLevel::Low);
        assert!(PriorityLevel::Low > PriorityLevel::Background);
    }

    #[test]
    fn test_priority_level_as_str() {
        assert_eq!(PriorityLevel::Critical.as_str(), "critical");
        assert_eq!(PriorityLevel::Background.as_str(), "background");
    }

    #[test]
    fn test_priority_level_from_str() {
        assert_eq!(PriorityLevel::from_str("high"), Some(PriorityLevel::High));
        assert_eq!(PriorityLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_priority_weight() {
        assert_eq!(PriorityLevel::Critical.weight(), 5);
        assert_eq!(PriorityLevel::Background.weight(), 1);
    }

    #[test]
    fn test_scheduling_policy_as_str() {
        assert_eq!(SchedulingPolicy::Fifo.as_str(), "fifo");
        assert_eq!(SchedulingPolicy::Priority.as_str(), "priority");
        assert_eq!(SchedulingPolicy::RoundRobin.as_str(), "round-robin");
        assert_eq!(SchedulingPolicy::Deadline.as_str(), "deadline");
    }

    #[test]
    fn test_scheduling_policy_from_str() {
        assert_eq!(SchedulingPolicy::from_str("fifo"), Some(SchedulingPolicy::Fifo));
        assert_eq!(SchedulingPolicy::from_str("priority"), Some(SchedulingPolicy::Priority));
        assert_eq!(SchedulingPolicy::from_str("round-robin"), Some(SchedulingPolicy::RoundRobin));
        assert_eq!(SchedulingPolicy::from_str("deadline"), Some(SchedulingPolicy::Deadline));
        assert_eq!(SchedulingPolicy::from_str("unknown"), None);
    }

    #[test]
    fn test_scheduling_policy_display() {
        assert_eq!(format!("{}", SchedulingPolicy::Fifo), "fifo");
    }

    #[test]
    fn test_scheduling_policy_default() {
        assert_eq!(SchedulingPolicy::default(), SchedulingPolicy::Priority);
    }
}
