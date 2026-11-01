"""Contract tests for generated verification evidence and failure behavior."""

import os
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GENERATOR = ROOT / ".github" / "scripts" / "generate-verification-report.py"
RUST_CASES = {
    "F1": ("f1_incomplete_transmission_expires_with_timeout", "f1_fault_bus_drops_final_packet_and_reassembler_times_out"),
    "F2": ("f2_reorder_dt_frames_returns_sequence_mismatch", "f2_fault_bus_reorders_frames_and_reassembler_rejects_sequence"),
    "F3": ("f3_abort_mid_stream_returns_reason", "f3_fault_bus_abort_is_typed_by_reassembler"),
    "F4": ("f4_oversized_payload_is_rejected", "f4_oversized_payload_is_rejected_before_any_bus_frame"),
    "F5": ("f5_twenty_bytes_in_two_packets_is_malformed", "f5_inconsistent_announcement_is_rejected_by_reassembler"),
    "F6": ("f6_duplicate_cm_resets_session_and_drops_first_message", "f6_duplicate_announcement_resets_session_and_completes_new_message"),
}
F1B_CASE = "f1b_dropped_middle_packet_is_reported_when_a_later_packet_arrives"
PYTHON_CASES = {
    "F1": ("test_f1_missing_final_packet_expires_at_sdk", "test_f1_missing_final_packet_expires_at_device"),
    "F2": ("test_f2_reordered_packets_return_sequence_error_at_sdk", "test_f2_reordered_packets_return_sequence_error_at_device"),
    "F3": ("test_f3_midstream_abort_preserves_reason_at_sdk", "test_f3_midstream_abort_preserves_reason_at_device"),
    "F4": ("test_f4_oversized_payload_is_typed_at_sdk", "test_f4_oversized_payload_is_typed_at_device"),
    "F5": ("test_f5_invalid_announcement_is_typed_at_sdk", "test_f5_invalid_announcement_is_typed_at_device"),
    "F6": ("test_f6_duplicate_announcement_resets_at_sdk", "test_f6_duplicate_announcement_resets_at_device"),
}


def run_generator(tmp_path: Path, *, omitted: set[str] = frozenset(), failed: set[str] = frozenset()):
    rust_path = tmp_path / "rust-tests.txt"
    junit_path = tmp_path / "report.xml"
    output = tmp_path / "evidence"
    rust_lines = []
    suite = ET.Element("testsuite")
    for names in RUST_CASES.values():
        for case in names:
            if case in omitted:
                continue
            state = "FAILED" if case in failed else "ok"
            rust_lines.append(f"test tests::{case} ... {state}")
    if F1B_CASE not in omitted:
        state = "FAILED" if F1B_CASE in failed else "ok"
        rust_lines.append(f"test tests::{F1B_CASE} ... {state}")
    for names in PYTHON_CASES.values():
        for case in names:
            if case in omitted:
                continue
            node = ET.SubElement(suite, "testcase", name=case)
            if case in failed:
                ET.SubElement(node, "failure", message="assertion failed")
    rust_path.write_text("\n".join(rust_lines), encoding="utf-8")
    ET.ElementTree(suite).write(junit_path, encoding="utf-8", xml_declaration=True)

    # Feed fixed run metadata so the success case proves the emitted link uses this CI run.
    env = os.environ.copy()
    env.update(
        GITHUB_SERVER_URL="https://github.com",
        GITHUB_REPOSITORY="example/j1939-lab",
        GITHUB_RUN_ID="12345",
    )
    result = subprocess.run(
        [sys.executable, str(GENERATOR), str(output), str(rust_path), str(junit_path)],
        capture_output=True,
        text=True,
        env=env,
        check=False,
    )
    return result, output


def test_report_marks_pass_and_uses_current_workflow_url(tmp_path):
    result, output = run_generator(tmp_path)

    assert result.returncode == 0, result.stderr
    report = (output / "verification-report.md").read_text(encoding="utf-8")
    assert "[workflow run](https://github.com/example/j1939-lab/actions/runs/12345)" in report
    assert "| F1 | `Timeout when no later packet arrives` | `PASS` | `PASS` | `PASS` | `PASS` |" in report
    assert "| F1b: middle packet dropped; later packet arrives | `SequenceMismatch` | `PASS` |" in report
    assert "20 bytes announced in 2 packets; 3 are required" in report
    assert (output / "status_f6_L4.txt").read_text(encoding="utf-8").strip() == "F6 L4: PASS"


def test_report_marks_failed_evidence_and_fails_generation(tmp_path):
    result, output = run_generator(
        tmp_path,
        failed={
            "f2_fault_bus_reorders_frames_and_reassembler_rejects_sequence",
            "test_f3_midstream_abort_preserves_reason_at_sdk",
        },
    )

    assert result.returncode == 1
    report = (output / "verification-report.md").read_text(encoding="utf-8")
    assert "| F2 | `SequenceMismatch` | `PASS` | `FAIL` | `PASS` | `PASS` |" in report
    assert "| F3 | `Aborted(3)` | `PASS` | `PASS` | `FAIL` | `PASS` |" in report


def test_report_marks_missing_scenario_and_fails_generation(tmp_path):
    result, output = run_generator(
        tmp_path,
        omitted={
            "test_f1_missing_final_packet_expires_at_device",
            F1B_CASE,
        },
    )

    assert result.returncode == 1
    report = (output / "verification-report.md").read_text(encoding="utf-8")
    assert "| F1 | `Timeout when no later packet arrives` | `PASS` | `PASS` | `PASS` | `MISSING` |" in report
    assert (output / "status_f1_L4.txt").read_text(encoding="utf-8").strip() == "F1 L4: MISSING"
    assert "| F1b: middle packet dropped; later packet arrives | `SequenceMismatch` | `MISSING` |" in report
    assert (output / "status_f1b_L1.txt").read_text(encoding="utf-8").strip() == "F1b L1: MISSING"
