use serde::{Deserialize, Serialize};
use crate::rewriting::egraph::ClassId;
use crate::language::symbol::Symbol;
use super::Literal;

/// An expression which consists of concrete elements as well as class IDs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MixedExpression {
    Literal(Literal),
    Symbol(Symbol<MixedExpression>),
    Class(ClassId),
}
