//! Integer Linear Programming (ILP) helpers.
//!
//! This module provides utilities for creating and solving ILP problems
//! using the `good_lp` library.

use good_lp::*;
use good_lp::solvers::coin_cbc::CoinCbcProblem;
use nalgebra::{DMatrix, DVector};

/// Creates an ILP problem for the given matrix A and vector d.
///
/// The problem is formulated as:
/// - minimize: 1^T x (sum of all components of x)
/// - subject to: Ax = d
/// - x >= 0
/// - x is integer-valued
///
/// This function returns a solver model that is ready to be solved,
/// along with the variable vector x.
/// All constraints are already added to the model.
/// The CBC solver is configured to suppress output by default.
///
/// # Arguments
///
/// * `a` - The constraint matrix A (dimensions: m x n)
/// * `d` - The right-hand side vector d (dimensions: m)
///
/// # Returns
///
/// Returns a tuple `(model, variables)` where:
/// - `model` is a `CoinCbcProblem` ready to be solved with `.solve()`
/// - `variables` is a `Vec<Variable>` containing the decision variables x
///
/// # Panics
///
/// Panics if the dimensions of A and d don't match (A.nrows() must equal d.len()).
///
/// # Example
///
/// ```rust
/// use nalgebra::{DMatrix, DVector};
/// use verbum::rewriting::ilp::create_ilp_problem;
/// use good_lp::{Solution, SolverModel};
///
/// // Create a simple problem: minimize x1 + x2 subject to x1 + x2 = 5
/// let a = DMatrix::from_row_slice(1, 2, &[1, 1]);
/// let d = DVector::from_vec(vec![5]);
///
/// let (model, vars) = create_ilp_problem(&a, &d);
/// let solution = model.solve();
///
/// if let Ok(sol) = solution {
///     let objective_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
///     println!("Solution found with objective value: {}", objective_value);
/// }
/// ```
pub fn create_ilp_problem(
    a: &DMatrix<i32>,
    d: &DVector<i32>,
) -> (CoinCbcProblem, Vec<Variable>) {
    let m = a.nrows();
    let n = a.ncols();

    assert_eq!(
        m,
        d.len(),
        "Matrix A has {} rows but vector d has {} elements. Dimensions must match.",
        m,
        d.len()
    );

    // Create problem variables
    let mut vars = ProblemVariables::new();
    let x: Vec<Variable> = (0..n)
        .map(|_| vars.add(variable().integer().min(0)))
        .collect();

    // Create objective: minimize 1^T x (sum of all x components)
    let objective: Expression = x.iter().copied().sum();

    // Start building the problem
    let mut problem = vars.minimise(objective).using(coin_cbc);

    // Suppress CBC solver output
    problem.set_parameter("log", "0");

    // Add constraints: Ax = d
    for i in 0..m {
        // Build the left-hand side of the i-th constraint: A[i,0]*x[0] + A[i,1]*x[1] + ...
        let mut constraint_expr = Expression::from(0);
        for j in 0..n {
            let coeff = a[(i, j)];
            if coeff != 0 {
                constraint_expr = constraint_expr + coeff * x[j];
            }
        }

        // Add the constraint: constraint_expr == d[i]
        problem = problem.with(constraint!(constraint_expr == d[i]));
    }

    (problem, x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{DMatrix, DVector};

    #[test]
    fn test_create_ilp_problem_simple() {
        // Simple problem: minimize x1 + x2 subject to x1 + x2 = 5
        let a = DMatrix::from_row_slice(1, 2, &[1, 1]);
        let d = DVector::from_vec(vec![5]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok(), "Expected feasible solution");

        let sol = solution.unwrap();
        // The optimal solution should have objective value 5 (e.g., x1=0, x2=5 or x1=5, x2=0 or any combination)
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        assert_eq!(obj_value, 5.0);
    }

    #[test]
    fn test_create_ilp_problem_multiple_constraints() {
        // Problem with multiple constraints:
        // minimize x1 + x2
        // subject to:
        //   x1 + x2 = 10
        //   x1 - x2 = 2
        // Solution: x1 = 6, x2 = 4, objective = 10
        let a = DMatrix::from_row_slice(2, 2, &[1, 1, 1, -1]);
        let d = DVector::from_vec(vec![10, 2]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok());
        let sol = solution.unwrap();
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        assert_eq!(obj_value, 10.0);
    }

    #[test]
    fn test_create_ilp_problem_three_variables() {
        // Problem: minimize x1 + x2 + x3
        // subject to: x1 + 2*x2 + 3*x3 = 12
        // Many solutions exist; optimal should minimize sum
        let a = DMatrix::from_row_slice(1, 3, &[1, 2, 3]);
        let d = DVector::from_vec(vec![12]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok());
        let sol = solution.unwrap();
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        // Optimal solution is x1=0, x2=0, x3=4, giving objective = 4
        assert_eq!(obj_value, 4.0);
    }

    #[test]
    fn test_create_ilp_problem_infeasible() {
        // Infeasible problem:
        // x1 + x2 = 5
        // x1 + x2 = 10
        // (contradictory constraints)
        let a = DMatrix::from_row_slice(2, 2, &[1, 1, 1, 1]);
        let d = DVector::from_vec(vec![5, 10]);

        let (model, _vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        // This should be infeasible
        assert!(solution.is_err(), "Expected infeasible solution");
    }

    #[test]
    fn test_create_ilp_problem_zero_vector() {
        // Problem: minimize x1 + x2 subject to x1 + x2 = 0
        // Solution: x1 = 0, x2 = 0 (since x >= 0)
        let a = DMatrix::from_row_slice(1, 2, &[1, 1]);
        let d = DVector::from_vec(vec![0]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok());
        let sol = solution.unwrap();
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        assert_eq!(obj_value, 0.0);
    }

    #[test]
    fn test_create_ilp_problem_with_zeros_in_matrix() {
        // Problem with zeros in the matrix:
        // minimize x1 + x2 + x3
        // subject to:
        //   x1 + 0*x2 + x3 = 5
        //   0*x1 + x2 + 0*x3 = 3
        // Solution: x1=0, x2=3, x3=5, objective = 8
        let a = DMatrix::from_row_slice(2, 3, &[1, 0, 1, 0, 1, 0]);
        let d = DVector::from_vec(vec![5, 3]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok());
        let sol = solution.unwrap();
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        assert_eq!(obj_value, 8.0);
    }

    #[test]
    fn test_create_ilp_problem_larger_system() {
        // Larger system with 4 variables and 3 constraints
        // minimize x1 + x2 + x3 + x4
        // subject to:
        //   x1 + x2 + x3 + x4 = 10
        //   2*x1 + x2 = 4
        //   x3 + 2*x4 = 6
        let a = DMatrix::from_row_slice(3, 4, &[1, 1, 1, 1, 2, 1, 0, 0, 0, 0, 1, 2]);
        let d = DVector::from_vec(vec![10, 4, 6]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok());
        let sol = solution.unwrap();
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        assert_eq!(obj_value, 10.0);
    }

    #[test]
    #[should_panic(expected = "Dimensions must match")]
    fn test_create_ilp_problem_mismatched_dimensions() {
        // Matrix A has 2 rows but vector d has 3 elements - should panic
        let a = DMatrix::from_row_slice(2, 3, &[1, 2, 3, 4, 5, 6]);
        let d = DVector::from_vec(vec![1, 2, 3]);

        create_ilp_problem(&a, &d);
    }

    #[test]
    fn test_create_ilp_problem_single_variable() {
        // Simplest case: minimize x subject to x = 7
        let a = DMatrix::from_row_slice(1, 1, &[1]);
        let d = DVector::from_vec(vec![7]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok());
        let sol = solution.unwrap();
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        assert_eq!(obj_value, 7.0);
    }

    #[test]
    fn test_create_ilp_problem_negative_coefficients() {
        // Problem with negative coefficients:
        // minimize x1 + x2
        // subject to: x1 - x2 = 4
        // Solution: x1 = 4, x2 = 0, objective = 4
        let a = DMatrix::from_row_slice(1, 2, &[1, -1]);
        let d = DVector::from_vec(vec![4]);

        let (model, vars) = create_ilp_problem(&a, &d);
        let solution = model.solve();

        assert!(solution.is_ok());
        let sol = solution.unwrap();
        let obj_value: f64 = vars.iter().map(|&v| sol.value(v)).sum();
        assert_eq!(obj_value, 4.0);
    }
}
