use crate::com::{
    Checker, ast,
    ir::{self, Constructor},
    loc::Span,
};

impl Checker<'_, '_> {
    pub fn check_match(
        &mut self,
        b: &ast::MatchBranch,
        span: Span,
    ) -> (ir::Branch, ir::TypeID, bool) {
        let result_type = self.create_fresh_type(Some(span));

        let (scrut, scrut_type) = self.check_expression(&b.scrutinee);
        let scrut_var = ir::VariableID::dummy();

        let mut cases = Vec::new();
        for case in &b.cases {
            let pattern = self.check_pattern(&case.pattern);

            self.open_scope(false);

            let (pattern, pattern_type) = self.declare_pattern(&pattern, false);
            self.unify(scrut_type, pattern_type, &[]);

            let (val, val_type) = self.check_expression(&case.value);
            self.unify(val_type, result_type, &[]);

            self.close_scope();

            let lhs = vec![MatchTest(scrut_var, pattern)];
            let rhs = MatchRhs(val, vec![]);
            cases.push((lhs, rhs));
        }

        let (decision, is_exhaustive) = self.build_decision_tree(cases);

        (
            ir::Branch::Match {
                scrutinee_var: scrut_var,
                scrutinee: Box::new(scrut),
                decision: Box::new(decision),
            },
            result_type,
            is_exhaustive,
        )
    }

    // algorithm based on: https://julesjacobs.com/notes/patternmatching/patternmatching.pdf
    // and with help from this implementation: https://gitlab.com/yorickpeterse/pattern-matching-in-rust/-/tree/main/jacobs2021
    fn build_decision_tree(&mut self, mut cases: MatchProblem) -> (ir::Decision, bool) {
        // find tests against irrefutable patterns and move them as variable let bindings
        for (lhs, MatchRhs(_, stmts)) in &mut cases {
            let (irrefutables, refutables) = std::mem::take(lhs)
                .into_iter()
                .partition(|MatchTest(_, pat)| pat.is_exhaustive());

            // retain tests that are refutable and not missing
            *lhs = refutables;
            lhs.retain(|MatchTest(_, pat)| !matches!(pat, ir::Pattern::Missing));

            for MatchTest(x, pat) in irrefutables {
                // push let <pat> = <x> on the rhs
                stmts.push(ir::Stmt::Let {
                    lhs: pat,
                    rhs: ir::Expr::Var { id: x },
                    is_concrete: true,
                    solutions: vec![],
                });
            }
        }

        // no cases means the pattern matching is non-exhaustive
        let Some((first_lhs, _)) = cases.first() else {
            return (ir::Decision::Failure, false);
        };

        // this case has no test, it always passes at this point
        if first_lhs.is_empty() {
            let MatchRhs(expr, stmts) = cases.swap_remove(0).1;
            return (
                ir::Decision::Success {
                    stmts: stmts.into(),
                    result: Box::new(expr),
                },
                true,
            );
        }

        // select a test in the first case
        let &MatchTest(checked_variable, ref checked_pattern) =
            Self::select_match_test(first_lhs, &cases);

        let constructor = checked_pattern.constructor();
        let variations = self.get_all_variations(&constructor);

        match variations {
            Some(variations) => {
                // build a match subproblem for each variation of the constructor
                let mut all_variants_found = true;
                let mut current_subproblem = cases;
                let mut decisions = Vec::new();
                for variant_constructor in variations {
                    let (pat, ok, success, failure) = self.create_subproblem(
                        checked_variable,
                        variant_constructor,
                        current_subproblem,
                    );

                    current_subproblem = failure;
                    all_variants_found &= ok;

                    let variant_decision = self.build_decision_tree(success);
                    decisions.push((pat, variant_decision));
                }

                // if a variation is missing, ensure that it is handled in the failure subproblem
                let (mut final_decision, mut is_exhaustive) = if all_variants_found {
                    (ir::Decision::Failure, true)
                } else {
                    self.build_decision_tree(current_subproblem)
                };

                for (pat, (decision, decision_exhaustive)) in decisions {
                    is_exhaustive &= decision_exhaustive;
                    final_decision = ir::Decision::Test {
                        tested_var: checked_variable,
                        pattern: Box::new(pat),
                        success: Box::new(decision),
                        failure: Box::new(final_decision),
                    };
                }

                (final_decision, is_exhaustive)
            }
            _ => {
                // in this case, there are an infinite number of cases for the current constructor
                // simply calculate subproblems based on this constructor check
                // the failure subproblem will check whether all possibilities are handled or not
                let (pat, _, success, failure) =
                    self.create_subproblem(checked_variable, constructor, cases);

                let (success_decision, success_exhaustive) = self.build_decision_tree(success);
                let (failure_decision, failure_exhaustive) = self.build_decision_tree(failure);

                (
                    ir::Decision::Test {
                        tested_var: checked_variable,
                        pattern: Box::new(pat),
                        success: Box::new(success_decision),
                        failure: Box::new(failure_decision),
                    },
                    success_exhaustive && failure_exhaustive,
                )
            }
        }
    }

    // None means an infinite number of variations for a given constructor
    // for example: the set of string patterns is infinite
    // but for a bool(_) constructor, the two cases are bool(true) and bool(false)
    pub fn get_all_variations(&self, c: &Constructor) -> Option<Vec<Constructor>> {
        use ir::Constructor as C;
        match c {
            C::Missing => None,
            C::Int(_) => None,
            C::Float(_) => None,
            C::String(_) => None,
            C::Bool(_) => Some(vec![C::Bool(true), C::Bool(false)]),
            C::Tuple(n) => Some(vec![C::Tuple(*n)]),
            C::Variant(id, _) => {
                let info = self.entities.get_union_info(*id);
                let variant_count = info.variant_count();
                let variations = (0..variant_count).map(|tag| C::Variant(*id, tag)).collect();
                Some(variations)
            }
            C::Record(id) => Some(vec![C::Record(*id)]),
        }
    }

    // (test pattern, any successful, success subproblem, failure subproblem)
    fn create_subproblem(
        &mut self,
        variable: ir::VariableID,
        constructor: ir::Constructor,
        cases: MatchProblem,
    ) -> (ir::Pattern, bool, MatchProblem, MatchProblem) {
        // create temporary variables for each subpattern in the chosen test
        let (test_pattern, inner_variables) = self.build_constructor_pattern(&constructor);

        // generate two new pattern matching subproblems
        let mut any_successful = false;
        let mut success_match_cases = Vec::new();
        let mut failure_match_cases = Vec::new();
        for (lhs, rhs) in cases {
            // find tests whose checked variable is the one being tested here (x)
            let (mut x_tests, mut new_tests): (Vec<_>, Vec<_>) =
                lhs.into_iter().partition(|MatchTest(x, _)| *x == variable);

            let Some(x_test) = x_tests.pop() else {
                // no test for x in this case, it could be reached
                // independently of the result of the test.
                // so we need to add it to both subproblems
                success_match_cases.push((new_tests.clone(), rhs.clone()));
                failure_match_cases.push((new_tests.clone(), rhs.clone()));
                continue;
            };

            // invariant(uniqueness): there is at most one test for x
            // so we should have zero elements at this point (the test was popped)
            debug_assert_eq!(x_tests.len(), 0);
            let pat = &x_test.1;
            if pat.constructor() == constructor {
                // x is tested against the same constructor, so the test passes
                // we add this case to the success subproblem
                // and deconstruct the relevant pattern for x
                let pat_args = pat.constructor_args();
                for (x_i, pat_i) in inner_variables.iter().zip(pat_args) {
                    new_tests.push(MatchTest(*x_i, pat_i));
                }
                success_match_cases.push((new_tests, rhs));
                any_successful = true;
            } else {
                // x is tested against another constructor
                // so we just move this case to the failure subproblem
                new_tests.push(x_test);
                failure_match_cases.push((new_tests, rhs));
            }
        }

        (
            test_pattern,
            any_successful,
            success_match_cases,
            failure_match_cases,
        )
    }

    // returns (binding pattern for given constructor, binding args)
    fn build_constructor_pattern(
        &mut self,
        cons: &ir::Constructor,
    ) -> (ir::Pattern, Vec<ir::VariableID>) {
        use ir::Constructor as C;
        use ir::Pattern as P;
        match cons {
            C::Missing => (P::Missing, vec![]),
            C::Int(n) => (P::Int(*n), vec![]),
            C::Float(f) => (P::Float(*f), vec![]),
            C::String(s) => (P::String(s.clone()), vec![]),
            C::Bool(b) => (P::Bool(*b), vec![]),
            C::Tuple(n) => {
                let (arg_patterns, arg_ids) = self.create_binding_patterns(*n);
                (P::Tuple(arg_patterns.into()), arg_ids)
            }
            C::Variant(id, tag) => {
                let (_, info) = self.entities.get_union_variant_info(*id, *tag);
                match info.arity() {
                    Some(n) => {
                        let (arg_patterns, arg_ids) = self.create_binding_patterns(n);
                        (P::Variant(*id, *tag, Some(arg_patterns.into())), arg_ids)
                    }
                    None => (P::Variant(*id, *tag, None), vec![]),
                }
            }
            C::Record(id) => {
                let info = self.entities.get_record_info(*id);
                let (arg_patterns, arg_ids) = self.create_binding_patterns(info.fields.len());
                (P::Record(*id, arg_patterns.into()), arg_ids)
            }
        }
    }

    fn create_binding_patterns(&mut self, n: usize) -> (Vec<ir::Pattern>, Vec<ir::VariableID>) {
        let arg_ids = (0..n).map(|_| ir::VariableID::dummy()).collect::<Vec<_>>();
        let arg_patterns = arg_ids.iter().map(|id| ir::Pattern::Binding(*id)).collect();
        (arg_patterns, arg_ids)
    }

    // select a test clause within a case
    fn select_match_test<'a>(
        test: &'a [MatchTest],
        _cases: &[(Vec<MatchTest>, MatchRhs)],
    ) -> &'a MatchTest {
        &test[0]
    }
}

type MatchProblem = Vec<(Vec<MatchTest>, MatchRhs)>;

#[derive(Debug, Clone)]
struct MatchRhs(ir::Expr, Vec<ir::Stmt>);

#[derive(Debug, Clone)]
struct MatchTest(ir::VariableID, ir::Pattern);
