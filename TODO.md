- Should implement SimpleSaturator and Directed Saturator as wrappers for ScheduledSaturator
- Add creation of rule dependency graph
- Add TRS reduction to cost-equivalent TRS without zero cost rules?
- Add target expression guessing in e-graphs
- Add directed saturation (direction of the target expression, like A*)
- Try creating the target expression using Monte-Carlo

To consider in directed saturation:
- Why is Directed saturation by sorting faster than regular saturation when the e-graph is saturated anyways!?
- When a single e-class is a child of many expressions, it contributes a lot to the cost, this should be considered.
- if a cheapening rewrite rule is of the form f(g(...)) -> h(...) and the egraph has form g(...),
  should we look to make it of the form f(g(...))? what about when e-graph is of the form f(...) (no g inside)?
- Formulate clearly: creating target expressions

Agent chores:
- Add analysis to `dot`
- Add Scheduler trait with `next_rule` method.
- Make it so that languages are in a different JSON, together with costs?
- Add a new extractor implemetnation for Analysis implementing `LocalCost`,
  such that it does not calculate the costs from the beginning but takes them from the analysis instead.
  This will probably run in `O(1)`.
- Add more expressions in main benchmarking
- make EGraphs not generic over analysis? Analysis dynamic, additionally AnalysisManager?
   (then it would be possible to have one `LocalCost` whose manager keeps hashmap of symbol costs and exposes the `make` method)
  
