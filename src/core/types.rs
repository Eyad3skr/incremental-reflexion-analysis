use std::fmt;

// enums + shared types
pub type NodeId = u32;
pub type EdgeId = u32;
pub type Counter = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubgraphKind {
    Architecture,
    Implementation,
    Propagated,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeKind(String);

impl EdgeKind {
    // Predefined edge kinds
    pub const CONTAINS: &'static str = "contains";
    pub const CALLS: &'static str = "calls";
    pub const DEPENDS_ON: &'static str = "depends_on";

    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }

    // ergonomic constructors (so we can write EdgeKind::from(...))
    pub fn contains() -> Self { Self::new(Self::CONTAINS) }
    pub fn calls() -> Self { Self::new(Self::CALLS) }
    pub fn depends_on() -> Self { Self::new(Self::DEPENDS_ON) }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

//Conversions
impl From<&str> for EdgeKind {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for EdgeKind {
    fn from(s: String) -> Self {
        Self(s)
    }
}

//log / print
impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

//can pass &edge_kind into APIs expecting AsRef<str>
impl AsRef<str> for EdgeKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKind {
    ArchitectureNode,
    ImplementationNode,
    DatastoreNode,
    ServiceNode,
    UINode,
    ModuleNode,
    ClassNode,
    PackageNode,
    FunctionNode,
    //if we need open-ended node tags too
    Custom(String),
}

impl NodeKind {
    //create a custom node kind from anything that can become a String
    //for example: NodeKind::Custom("Repository".to_string())
    pub fn custom<S: Into<String>>(s: S) -> Self {
        NodeKind::Custom(s.into())
    }
}

