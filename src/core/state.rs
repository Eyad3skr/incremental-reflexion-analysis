// convergent, divergent, etc..
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EdgeState {
    Undefined, //we don't know yet/missing data
    Specified, //edge exists in Architecture spec 
    Convergent, //spec says it should exist AND code contains a matching edge (happy path)
    Absent, //spec says it should exist BUT code doesn't contain it 
    AllowedAbsent, //spec says it's optional, if it is missing it is okay
    Allowed, //spec doesn't mention it explicitly, but the spec allows it (allowed edge list) 
    Divergent, //code contains and edge that isn't specified and not allowed by rules/spec 
    Unmapped, //we can't compare because mapping is missing
}

impl EdgeState {
    //violation -> (absent, divergent)
    //not a violation -> (convergent, allowed, allowedAbsent)
    //neither {analysis incomplete / undecided} -> (undefined, unmapped, specified)
    //violations -> architectural debt
    //undefined/unmapped -> tooling or modeling debt
    pub fn is_violation(&self) -> bool {
        matches!(self, EdgeState::Absent | EdgeState::Divergent)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, EdgeState::Undefined | EdgeState::Unmapped | EdgeState::Specified)
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, EdgeState::Allowed | EdgeState::AllowedAbsent | EdgeState::Convergent)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NodeState {
    Mapped, //implementation node successfully maps to an architecture node 
    Unmapped, //implementation node exists but has no mapping to the architecture 
    SpecifiedOnly, //architecture node exists but has no mapped implementation node 
    Undefined, //node has not been classified yet OR pipeline not run
}

impl NodeState {
    pub fn is_problem(&self) -> bool {
        matches!(self, NodeState::Unmapped | NodeState::SpecifiedOnly)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, NodeState::Undefined)
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, NodeState::Mapped)
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeState;

    #[test]
    fn is_violation_true_for_absent_and_divergent() {
        assert!(EdgeState::Absent.is_violation());
        assert!(EdgeState::Divergent.is_violation());
    }

    #[test]
    fn is_violation_false_for_non_violations() {
        assert!(!EdgeState::Undefined.is_violation());
        assert!(!EdgeState::Specified.is_violation());
        assert!(!EdgeState::Convergent.is_violation());
        assert!(!EdgeState::AllowedAbsent.is_violation());
        assert!(!EdgeState::Allowed.is_violation());
        assert!(!EdgeState::Unmapped.is_violation());
    }
}
