use super::Literal;
use crate::language::symbol::Symbol;
use crate::rewriting::egraph::ClassId;
use serde::{Deserialize, Serialize};

/// An expression which consists of concrete elements as well as class IDs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MixedExpression {
    Literal(Literal),
    Symbol(Symbol<MixedExpression>),
    Class(ClassId),
}
