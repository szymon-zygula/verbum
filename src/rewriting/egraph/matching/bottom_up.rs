use super::Matcher;

pub struct BottomUpMatcher;

impl BottomUpMatcher {}

impl Matcher for BottomUpMatcher {
    fn try_match(
        &self,
        egraph: &crate::rewriting::egraph::EGraph,
        expression: &crate::language::expression::Expression,
    ) -> Vec<super::EGraphMatch> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
