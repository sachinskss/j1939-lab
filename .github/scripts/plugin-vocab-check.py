from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2] / "python" / "pytest_j1939"
forbidden = re.compile(r"\b(?:PGN|BAM|CAN|J1939)\b", re.IGNORECASE)
violations = []
for path in root.rglob("*.py"):
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if forbidden.search(line):
            violations.append(f"{path}:{number}: forbidden protocol vocabulary")
if violations:
    print("\n".join(violations), file=sys.stderr)
    raise SystemExit(1)
print("plugin vocabulary check passed")
