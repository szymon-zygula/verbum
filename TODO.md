# NOW
- More benchmarks!
- Use dependency graphs

# Other
- Add target expression guessing in e-graphs
- Add directed saturation (direction of the target expression, like A*)
- Try creating the target expression using Monte-Carlo
- Add a new extractor implemetnation for Analysis implementing `LocalCost`,
  such that it does not calculate the costs from the beginning but takes them from the analysis instead.
  This will probably run in `O(1)`.
  

To consider in directed saturation:
- Why is Directed saturation by sorting faster than regular saturation when the e-graph is saturated anyways!?
- Formulate clearly: creating target expressions
- When a single e-class is a child of many expressions, it contributes a lot to the cost, this should be considered.
- if a cheapening rewrite rule is of the form f(g(...)) -> h(...) and the egraph has form g(...),
  should we look to make it of the form f(g(...))? what about when e-graph is of the form f(...) (no g inside)?

Agent chores:
- Reduce test repetition
- Make it so that languages are in a different JSON, together with costs?
- Add more expressions in main benchmarking
