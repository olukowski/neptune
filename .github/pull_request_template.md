## Scope

State/algorithm and pinned specification section:
<!-- Prefer one state per PR. Small shared infrastructure PRs are also welcome. -->

## Behavior and transitions

- Branches covered (including EOF and parse errors):
- Transitions to still-unimplemented states:
- Input consumption / reconsumption effects:

## Evidence

- Focused tests:
- Upstream conformance cases executed / passed / failed / blocked:
<!-- Inventory checks are not conformance tests. Do not count blocked cases as passed. -->
- Commands run:

## Review checklist

- [ ] Specification branches are traceable to code and tests.
- [ ] Unimplemented targets fail explicitly, never masquerading as parse errors or EOF.
- [ ] Upstream expected results were not changed to match our implementation.
- [ ] No unmeasured optimization or unrelated refactoring.
- [ ] Documentation/coverage status reflects what actually works.
