"""Build a scenario matrix from Rust output and pytest's machine-readable JUnit."""

from __future__ import annotations

import os
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "verification-evidence"
RUST_LOG = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "rust-tests.txt"
JUNIT = Path(sys.argv[3]) if len(sys.argv) > 3 else ROOT / "report.xml"

# Exact test names make a missing scenario visible instead of silently treating
# a green aggregate job as evidence for every row.
RUST_CASES = {
    "F1": ("f1_incomplete_transmission_expires_with_timeout", "f1_fault_bus_drops_final_packet_and_reassembler_times_out"),
    "F2": ("f2_reorder_dt_frames_returns_sequence_mismatch", "f2_fault_bus_reorders_frames_and_reassembler_rejects_sequence"),
    "F3": ("f3_abort_mid_stream_returns_reason", "f3_fault_bus_abort_is_typed_by_reassembler"),
    "F4": ("f4_oversized_payload_is_rejected", "f4_oversized_payload_is_rejected_before_any_bus_frame"),
    "F5": ("f5_twenty_bytes_in_two_packets_is_malformed", "f5_inconsistent_announcement_is_rejected_by_reassembler"),
    "F6": ("f6_duplicate_cm_resets_session_and_drops_first_message", "f6_duplicate_announcement_resets_session_and_completes_new_message"),
}
PYTHON_CASES = {
    "F1": ("test_f1_missing_final_packet_expires_at_sdk", "test_f1_missing_final_packet_expires_at_device"),
    "F2": ("test_f2_reordered_packets_return_sequence_error_at_sdk", "test_f2_reordered_packets_return_sequence_error_at_device"),
    "F3": ("test_f3_midstream_abort_preserves_reason_at_sdk", "test_f3_midstream_abort_preserves_reason_at_device"),
    "F4": ("test_f4_oversized_payload_is_typed_at_sdk", "test_f4_oversized_payload_is_typed_at_device"),
    "F5": ("test_f5_invalid_announcement_is_typed_at_sdk", "test_f5_invalid_announcement_is_typed_at_device"),
    "F6": ("test_f6_duplicate_announcement_resets_at_sdk", "test_f6_duplicate_announcement_resets_at_device"),
}
EXPECTED = {
    "F1": "Timeout when no later packet arrives",
    "F2": "SequenceMismatch",
    "F3": "Aborted(3)",
    "F4": "PayloadTooLarge(2000)",
    "F5": "MalformedCm: 20 bytes announced in 2 packets; 3 are required",
    "F6": "New session completes; prior session data is discarded",
}
F1B_CASE = "f1b_dropped_middle_packet_is_reported_when_a_later_packet_arrives"


def rust_status(log: str, case: str) -> str:
    match = re.search(rf"\b{re.escape(case)}\b[^\r\n]*", log)
    if match is None:
        return "MISSING"
    return "FAIL" if re.search(r"\bFAILED\b", match.group()) else "PASS"


def junit_status(root: ET.Element, case: str) -> str:
    matches = [node for node in root.iter("testcase") if node.get("name") == case]
    if not matches:
        return "MISSING"
    return "FAIL" if any(list(node) for node in matches) else "PASS"


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    rust = RUST_LOG.read_text(encoding="utf-8", errors="replace") if RUST_LOG.exists() else ""
    try:
        junit = ET.parse(JUNIT).getroot()
    except (ET.ParseError, OSError):
        junit = ET.Element("testsuites")

    statuses: dict[str, tuple[str, str, str, str]] = {}
    for scenario in RUST_CASES:
        l1_case, l2_case = RUST_CASES[scenario]
        l3_case, l4_case = PYTHON_CASES[scenario]
        statuses[scenario] = (
            rust_status(rust, l1_case),
            rust_status(rust, l2_case),
            junit_status(junit, l3_case),
            junit_status(junit, l4_case),
        )

    run_url = ""
    if os.environ.get("GITHUB_RUN_ID"):
        run_url = (
            f"{os.environ.get('GITHUB_SERVER_URL', 'https://github.com')}/"
            f"{os.environ.get('GITHUB_REPOSITORY', '')}/actions/runs/{os.environ['GITHUB_RUN_ID']}"
        )
    report_link = f"[workflow run]({run_url})" if run_url else "Local run (no hosted CI URL)"
    lines = [
        "# Verification report",
        "",
        "## Traceability Matrix",
        "",
        "This report is generated from named Rust test output and pytest JUnit XML.",
        "A missing test is reported as `MISSING`; it never becomes a pass by assumption.",
        "",
        "| Scenario | Expected outcome | L1 core | L2 bus | L3 SDK | L4 plugin | CI run |",
        "|---|---|---|---|---|---|---|",
    ]
    for scenario, row in statuses.items():
        lines.append(
            f"| {scenario} | `{EXPECTED[scenario]}` | "
            + " | ".join(f"`{state}`" for state in row)
            + f" | {report_link} |"
        )

    f1b_status = rust_status(rust, F1B_CASE)
    lines.append(
        f"| F1b: middle packet dropped; later packet arrives | `SequenceMismatch` | "
        f"`{f1b_status}` | N/A | N/A | N/A | {report_link} |"
    )

    for scenario, row in statuses.items():
        for layer, state in zip(("L1", "L2", "L3", "L4"), row):
            (OUT / f"status_{scenario.lower()}_{layer}.txt").write_text(
                f"{scenario} {layer}: {state}\n", encoding="utf-8"
            )
    (OUT / "status_f1b_L1.txt").write_text(f"F1b L1: {f1b_status}\n", encoding="utf-8")
    (OUT / "verification-report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    missing = [(scenario, layer) for scenario, row in statuses.items()
               for layer, state in zip(("L1", "L2", "L3", "L4"), row) if state == "MISSING"]
    failed = [(scenario, layer) for scenario, row in statuses.items()
              for layer, state in zip(("L1", "L2", "L3", "L4"), row) if state == "FAIL"]
    if f1b_status == "MISSING":
        missing.append(("F1b", "L1"))
    elif f1b_status == "FAIL":
        failed.append(("F1b", "L1"))
    if missing or failed:
        print(f"verification evidence incomplete: missing={missing}, failed={failed}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
