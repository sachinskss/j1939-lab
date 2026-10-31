import os
from pathlib import Path

import pytest

from j1939_sdk import connect


@pytest.fixture
async def bus():
    root = Path(__file__).resolve().parents[2]
    binary = root / "target" / "debug" / "bam_demo"
    if os.name == "nt":
        binary = binary.with_suffix(".exe")
    client = await connect(str(binary))
    try:
        yield client
    finally:
        await client.close()
