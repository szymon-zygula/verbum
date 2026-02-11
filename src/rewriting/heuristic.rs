//! Heuristic functions for guiding term rewriting.
//!
//! This module provides traits and implementations for heuristics that estimate
//! the lower bound distance between expressions in a term rewriting system.
//!
//! # Mathematical Background
//!
//! The ILP heuristic implements the following formula:
//!
//! ```text
//! h(e) = max_{v ∈ V_{e,e'}} min_{α ∈ Ω^e_v} max_{ω ∈ Ω^{e'}_v} θ(M_T, a(ω) - a(α))
//! ```
//!
//! Where:
//! - `V_{e,e'}` is the set of variables appearing in `e` or `e'`
//! - `Ω^e_v` is the set of all paths in `e` from root to variable `v`
//! - `a(p)` is the abelianized vector of path `p`
//! - `θ(A, d)` solves: minimize 1^T x subject to Ax = d, x ≥ 0, x integer
//! - `M_T` is the abelianized matrix of the TRS
//!
//! Conventions:
//! - `min_{x ∈ ∅} y = ∞`
//! - `max_{x ∈ ∅} y = 0`
//! - If the ILP has no solution, `θ(A, d) = ∞`
//!
//! # Examples
//!
//! Creating and using an ILP heuristic:
//!
//! ```rust,ignore
//! use verbum::{
//!     language::{Language, arities::Arities},
//!     macros::rules,
//!     rewriting::{
//!         heuristic::{Heuristic, AbelianPathHeuristic},
//!         system::TermRewritingSystem,
//!     },
//!     compact::SinglyCompact,
//! };
//! use std::collections::HashMap;
//!
//! let lang = Language::default()
//!     .add_symbol("+")
//!     .add_symbol("*");
//!
//! let mut arities_map = HashMap::new();
//! arities_map.insert(0, 2);
//! arities_map.insert(1, 2);
//! let arities = Arities::from(arities_map);
//!
//! let rules = rules!(lang;
//!     "(+ $0 $1)" => "(* $0 $1)"
//! );
//! let trs = TermRewritingSystem::new(lang.clone(), rules);
//!
//! let target = lang.parse("(* $0 $1)").unwrap();
//! let current = lang.parse("(+ $0 $1)").unwrap();
//!
//! let heuristic = AbelianPathHeuristic::new(&target, &trs, &arities);
//! let distance = heuristic.lower_bound_dist(&current);
//!
//! match distance {
//!     SinglyCompact::Finite(d) => println!("Distance: {}", d),
//!     SinglyCompact::Infinite => println!("Target unreachable"),
//! }
//! ```

use crate::compact::SinglyCompact;
use crate::language::{Language, arities::Arities, expression::{Expression, VariableId}};
use crate::rewriting::{
    ilp::create_ilp_problem,
    strings::{
        get_path_abelian_vectors_to_variables,
        rules_to_abelian_matrix, to_string_language, PathAbelianVector,
    },
    system::TermRewritingSystem,
};
use good_lp::{Solution, SolverModel, default_solver};
use nalgebra::DVector;
use std::collections::HashMap;

/// A heuristic function that provides a lower bound on the distance to a goal.
///
/// Heuristics are used to guide search algorithms by estimating how far an
/// expression is from a desired form.
pub trait Heuristic {
    /// Computes a lower bound on the distance from the given expression to the goal.
    ///
    /// # Arguments
    ///
    /// * `expression` - The expression to evaluate
    ///
    /// # Returns
    ///
    /// Returns a lower bound estimate of the distance, which may be infinite
    /// if the goal cannot be reached from this expression.
    fn lower_bound_dist(&self, expression: &Expression) -> SinglyCompact<u32>;
}

/// A factory for constructing heuristics.
///
/// The constructor trait allows creating heuristics that are specific to
/// a target expression and term rewriting system.
pub trait HeuristicConstructor {
    /// Constructs a heuristic for the given target expression and TRS.
    ///
    /// # Arguments
    ///
    /// * `expression` - The target expression (goal)
    /// * `trs` - The term rewriting system defining valid transformations
    ///
    /// # Returns
    ///
    /// Returns a boxed heuristic that can estimate distances to the target expression.
    fn construct(&self, expression: &Expression, trs: &TermRewritingSystem) -> Box<dyn Heuristic>;
}

/// A heuristic based on solving Integer Linear Programming (ILP) problems.
///
/// This heuristic computes a lower bound on the rewrite distance by:
/// 1. For each variable v in either expression
/// 2. For each path α in the current expression ending at v
/// 3. For each path ω in the target expression ending at v
/// 4. Solving an ILP to find the minimum number of rules needed to transform
///    the abelianized vector difference a(ω) - a(α)
/// 5. Taking the maximum over all variables of the minimum over all paths in the current
///    expression of the maximum over all paths in the target expression
///
/// This implements the mathematical formula:
/// h(e) = max_{v ∈ V_{e,e'}} min_{α ∈ Ω^e_v} max_{ω ∈ Ω^{e'}_v} θ(M_T, a(ω) - a(α))
///
/// where:
/// - V_{e,e'} is the set of variables appearing in e or e'
/// - Ω^e_v is the set of all paths in e from root to variable v
/// - a(p) is the abelianized vector of path p
/// - θ(A, d) is the solution to the ILP minimize 1^T x subject to Ax = d, x ≥ 0, x integer
/// - M_T is the abelianized matrix of T_s (the induced string rewriting system)
pub struct AbelianPathHeuristic {
    /// Grouped target paths by variable ID (precomputed for performance)
    target_by_var: HashMap<VariableId, Vec<PathAbelianVector>>,
    /// The abelianized matrix of the induced string rewriting system T_s
    abelian_matrix: nalgebra::DMatrix<i32>,
    /// The string language for computing paths
    string_lang: Language,
    /// Arities for the symbols
    arities: Arities,
    /// The original language
    lang: Language,
}

impl AbelianPathHeuristic {
    /// Creates a new ILP-based heuristic.
    ///
    /// # Arguments
    ///
    /// * `target_expr` - The target expression
    /// * `trs` - The term rewriting system
    /// * `arities` - Arities for the symbols in the language
    ///
    /// # Returns
    ///
    /// Returns a new `AbelianPathHeuristic` instance.
    pub fn new(target_expr: &Expression, trs: &TermRewritingSystem, arities: &Arities) -> Self {
        let lang = trs.language().clone();
        let string_lang = to_string_language(&lang, arities);
        
        // Precompute abelianized vectors for all paths in the target expression
        let target_paths = get_path_abelian_vectors_to_variables(
            target_expr,
            &lang,
            &string_lang,
            arities,
        );
        
        // Precompute target paths grouped by variable ID for performance
        let mut target_by_var: HashMap<VariableId, Vec<PathAbelianVector>> = HashMap::new();
        for path in &target_paths {
            target_by_var
                .entry(path.variable_id)
                .or_default()
                .push(path.clone());
        }
        
        // Convert TRS rules to induced string rewriting rules
        // M_T is the abelianized matrix of T_s (the induced string rewriting system)
        let induced_rules: Vec<_> = trs.rules()
            .iter()
            .flat_map(|rule| {
                crate::rewriting::strings::rule_to_induced_rules(
                    rule,
                    &lang,
                    &string_lang,
                    arities,
                )
            })
            .collect();
        
        // Compute the abelianized matrix from the induced rules
        let abelian_matrix = rules_to_abelian_matrix(&induced_rules, &string_lang);
        
        Self {
            target_by_var,
            abelian_matrix,
            string_lang,
            arities: arities.clone(),
            lang,
        }
    }
    
    /// Solves the ILP problem for a given difference vector.
    ///
    /// This method solves the Integer Linear Programming problem:
    /// ```text
    /// minimize: 1^T x (sum of all components)
    /// subject to: M_T x = diff_vector
    ///             x ≥ 0
    ///             x is integer-valued
    /// ```
    ///
    /// where M_T is the abelianized matrix of the TRS and x represents the number
    /// of times each rule is applied.
    ///
    /// # Arguments
    ///
    /// * `diff_vector` - The difference vector d = a(ω) - a(α) representing the
    ///   difference between target and current path abelianized vectors
    ///
    /// # Returns
    ///
    /// Returns:
    /// - `Finite(n)` where n is the optimal objective value (minimum sum of rule applications)
    /// - `Infinite` if the ILP problem is infeasible (target unreachable)
    ///
    /// # Special Cases
    ///
    /// - If there are no rules (empty TRS), returns `Finite(0)` if diff_vector is zero,
    ///   otherwise `Infinite`
    fn solve_ilp(&self, diff_vector: &DVector<i32>) -> SinglyCompact<u32> {
        if self.abelian_matrix.ncols() == 0 {
            // No rules available
            return if diff_vector.iter().all(|&x| x == 0) {
                SinglyCompact::Finite(0)
            } else {
                SinglyCompact::Infinite
            };
        }
        
        let (model, vars) = create_ilp_problem(&self.abelian_matrix, diff_vector, default_solver);
        
        match model.solve() {
            Ok(solution) => {
                let obj_value: f64 = vars.iter().map(|&v| solution.value(v)).sum();
                // Round to nearest integer (should already be integer due to ILP)
                SinglyCompact::Finite(obj_value.round() as u32)
            }
            Err(_) => SinglyCompact::Infinite,
        }
    }
}

impl Heuristic for AbelianPathHeuristic {
    fn lower_bound_dist(&self, expression: &Expression) -> SinglyCompact<u32> {
        // Get all paths in the current expression
        let current_paths = get_path_abelian_vectors_to_variables(
            expression,
            &self.lang,
            &self.string_lang,
            &self.arities,
        );
        
        // Group paths by variable ID
        let mut current_by_var: HashMap<VariableId, Vec<&PathAbelianVector>> = HashMap::new();
        for path in &current_paths {
            current_by_var
                .entry(path.variable_id)
                .or_default()
                .push(path);
        }
        
        // Get all variables from either expression
        let mut all_vars: Vec<VariableId> = current_by_var.keys().copied().collect();
        for &var_id in self.target_by_var.keys() {
            if !all_vars.contains(&var_id) {
                all_vars.push(var_id);
            }
        }
        
        let mut max_over_vars = SinglyCompact::Finite(0);
        
        // For each variable v in V_{e,e'}
        for var_id in all_vars {
            let current_paths_for_var = current_by_var.get(&var_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let target_paths_for_var = self.target_by_var.get(&var_id).map(|v| v.as_slice()).unwrap_or(&[]);
            
            let mut min_over_current = SinglyCompact::Infinite;
            
            // For each path α in Ω^e_v (paths in current expression)
            for current_path in current_paths_for_var {
                // Take maximum over target paths using iterator
                let max_over_target = target_paths_for_var
                    .iter()
                    .map(|target_path| {
                        // Compute difference: a(ω) - a(α)
                        let diff = &target_path.vector - &current_path.vector;
                        // Solve ILP: θ(M_T, diff)
                        self.solve_ilp(&diff)
                    })
                    .max()
                    .unwrap_or(SinglyCompact::Finite(0)); // Convention: max over empty set = 0
                
                // Take minimum over current paths
                if max_over_target < min_over_current {
                    min_over_current = max_over_target;
                }
            }
            
            // Short-circuit if min_over_current is infinite
            if min_over_current.is_infinite() {
                return SinglyCompact::Infinite;
            }
            
            // Take maximum over variables
            if min_over_current > max_over_vars {
                max_over_vars = min_over_current;
            }
        }
        
        max_over_vars
    }
}

/// Default constructor for abelian path heuristics.
///
/// This constructor creates `AbelianPathHeuristic` instances. It requires arities to be
/// provided when constructing the heuristic.
pub struct AbelianPathHeuristicConstructor {
    /// Arities for symbols in the language
    pub arities: Arities,
}

impl HeuristicConstructor for AbelianPathHeuristicConstructor {
    fn construct(&self, expression: &Expression, trs: &TermRewritingSystem) -> Box<dyn Heuristic> {
        Box::new(AbelianPathHeuristic::new(expression, trs, &self.arities))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::rules;
    use std::collections::HashMap;

    #[test]
    fn test_ilp_heuristic_identical_expressions() {
        let lang = Language::default().add_symbol("+").add_symbol("*");
        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2); // + has arity 2
        arities_map.insert(1, 2); // * has arity 2
        let arities = Arities::from(arities_map);

        let rules = rules!(lang;
            "(+ $0 $1)" => "(* $0 $1)"
        );
        let trs = TermRewritingSystem::new(lang.clone(), rules);

        let expr = lang.parse("(+ $0 $1)").unwrap();
        let heuristic = AbelianPathHeuristic::new(&expr, &trs, &arities);

        // Distance from an expression to itself should be 0
        let dist = heuristic.lower_bound_dist(&expr);
        assert_eq!(dist, SinglyCompact::Finite(0));
    }

    #[test]
    fn test_ilp_heuristic_simple_rewrite() {
        let lang = Language::default().add_symbol("+").add_symbol("*");
        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2);
        arities_map.insert(1, 2);
        let arities = Arities::from(arities_map);

        let rules = rules!(lang;
            "(+ $0 $1)" => "(* $0 $1)"
        );
        let trs = TermRewritingSystem::new(lang.clone(), rules);

        let target = lang.parse("(* $0 $1)").unwrap();
        let current = lang.parse("(+ $0 $1)").unwrap();
        
        let heuristic = AbelianPathHeuristic::new(&target, &trs, &arities);
        let dist = heuristic.lower_bound_dist(&current);
        
        // Should require at least 1 rule application
        assert!(dist.is_finite());
        if let SinglyCompact::Finite(d) = dist {
            assert!(d >= 1);
        }
    }

    #[test]
    fn test_ilp_heuristic_no_variables() {
        let lang = Language::default().add_symbol("+");
        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2);
        let arities = Arities::from(arities_map);

        let rules = rules!(lang;
            "(+ 1 2)" => "(+ 3 4)"
        );
        let trs = TermRewritingSystem::new(lang.clone(), rules);

        // Expressions with no variables
        let target = lang.parse("(+ 5 6)").unwrap();
        let current = lang.parse("(+ 7 8)").unwrap();
        
        let heuristic = AbelianPathHeuristic::new(&target, &trs, &arities);
        let dist = heuristic.lower_bound_dist(&current);
        
        // With no variables, distance should be 0 (convention: max over empty set = 0)
        assert_eq!(dist, SinglyCompact::Finite(0));
    }

    #[test]
    fn test_heuristic_constructor() {
        let lang = Language::default().add_symbol("+");
        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2);
        let arities = Arities::from(arities_map);

        let rules = rules!(lang;
            "(+ $0 $1)" => "(+ $1 $0)"
        );
        let trs = TermRewritingSystem::new(lang.clone(), rules);

        let constructor = AbelianPathHeuristicConstructor { arities };
        let expr = lang.parse("(+ $0 $1)").unwrap();
        
        let heuristic = constructor.construct(&expr, &trs);
        let dist = heuristic.lower_bound_dist(&expr);
        
        assert_eq!(dist, SinglyCompact::Finite(0));
    }
}
