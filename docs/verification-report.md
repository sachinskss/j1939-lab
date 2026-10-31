# Verification report

Status: all six Rust scenarios pass locally. The CI link remains pending until a remote run exists.
The expected result for each row is a typed core error or the documented reset behavior.
Detailed boundary, transition, frame-encoding, and bridge cases are listed in
[test-coverage.md](test-coverage.md).

| Scenario | Expected error | Result | CI link |
|---|---|---|---|
| F1: drop one TP.DT frame | `Timeout` | PASS locally | Pending |
| F2: reorder TP.DT frames | `SequenceMismatch` | PASS locally | Pending |
| F3: TP.CM abort mid-stream | `Aborted(3)` | PASS locally | Pending |
| F4: 2000-byte payload | `PayloadTooLarge(2000)` | PASS locally | Pending |
| F5: TP.CM with bad size | `MalformedCm` | PASS locally | Pending |
| F6: duplicate BAM announcement | First session resets; second completes | PASS locally | Pending |

F1 checks that a missing packet expires without a wall-clock read in the core.
F2 checks packet ordering and typed rejection. F3 checks that the abort reason is
preserved. F4 checks the protocol payload limit before transmission starts. F5
checks the announced packet count against the announced size. F6 checks that a
new announcement replaces an incomplete session from the same source.
