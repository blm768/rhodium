use rhodium::base;

pub enum NodeType {
    Operation,
    Integer,
}

trait Node {
    fn location(&self) -> base::SourceLocation;
    fn node_type() -> NodeType;
}
