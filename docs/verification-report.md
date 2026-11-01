# Verification report

CI generates the result table from named Rust test output and pytest JUnit XML.
The generated report and per-scenario status files are published in the
`verification-evidence` workflow artifact. This source document makes no test
result claim; missing test evidence causes the report job to fail.

Previous workflow reference supplied during review: [ci #1](https://github.com/sachinskss/j1939-lab/actions/runs/41246215720)
(green, 1m 8s). This historical link is not used as evidence for the current
source revision.

Scenario definitions and rationale are maintained in
[test-coverage.md](test-coverage.md). The generated artifact reports expected
outcomes and F1–F6 at L1 (core), L2 (simulation bus plus reassembler), L3 (SDK),
and L4 (pytest fixture) separately. It also reports F1b: a dropped middle
packet followed by later data returns `SequenceMismatch`; F1 is the separate
timeout case where no later packet arrives.
