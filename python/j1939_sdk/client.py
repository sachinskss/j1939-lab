"""Async client for the versioned line-delimited subprocess protocol."""

from __future__ import annotations

import asyncio
import base64
import json
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class Message:
    pgn: int
    source: int
    payload: bytes


class TpError(Exception):
    """Base error for protocol failures returned by the subprocess."""

    @property
    def variant(self) -> str:
        return type(self).__name__


class Timeout(TpError):
    pass


class SequenceMismatch(TpError):
    pass


class Aborted(TpError):
    pass


class PayloadTooLarge(TpError):
    pass


class MalformedCm(TpError):
    pass


class ProtocolError(TpError):
    pass


class InternalError(TpError):
    """The subprocess crashed or returned a response the SDK cannot parse."""


class Bus:
    def __init__(self, process: asyncio.subprocess.Process) -> None:
        self._process = process
        self._pending: asyncio.Queue[Message | BaseException] = asyncio.Queue()

    async def send_bam(
        self,
        pgn: int,
        source: int,
        payload: bytes,
        *,
        fault: dict[str, Any] | None = None,
    ) -> None:
        if self._process.stdin is None:
            raise ProtocolError("subprocess stdin is closed")
        request = {
            "version": "v1",
            "cmd": "send_bam",
            "pgn": pgn,
            "source": source,
            "payload": base64.b64encode(payload).decode("ascii"),
        }
        # Keep optional controls flat in v1 so the tiny Rust bridge can parse them
        # without embedding a second general-purpose JSON implementation.
        if fault:
            request.update(fault)
        self._process.stdin.write((json.dumps(request) + "\n").encode())
        await self._process.stdin.drain()
        await self._read_response()

    async def recv(self, timeout: float = 2.0) -> Message:
        try:
            result = await asyncio.wait_for(self._pending.get(), timeout)
        except TimeoutError as exc:
            raise asyncio.TimeoutError(f"no message arrived within {timeout:g} seconds") from exc
        if isinstance(result, BaseException):
            raise result
        return result

    async def close(self) -> None:
        if self._process.stdin and not self._process.stdin.is_closing():
            self._process.stdin.close()
        await self._process.wait()

    async def _read_response(self) -> None:
        if self._process.stdout is None:
            raise InternalError("subprocess stdout is closed")
        line = await self._process.stdout.readline()
        if not line:
            detail = "subprocess closed its output"
            if self._process.stderr is not None:
                detail += ": " + (await self._process.stderr.read()).decode(errors="replace")
            code = self._process.returncode
            raise InternalError(f"subprocess exited with code {code}: {detail}")
        try:
            response: dict[str, Any] = json.loads(line)
            if response.get("version", "v1") != "v1":
                raise InternalError("unsupported subprocess protocol version")
            if not response.get("ok"):
                error = response.get("error", {})
                kind = error.get("kind", "ProtocolError")
                detail = error.get("detail", "protocol request failed")
                error_type = {
                    "Timeout": Timeout,
                    "SequenceMismatch": SequenceMismatch,
                    "Aborted": Aborted,
                    "PayloadTooLarge": PayloadTooLarge,
                    "MalformedCm": MalformedCm,
                    "InternalError": InternalError,
                }.get(kind, ProtocolError)
                await self._pending.put(error_type(detail))
                return
            item = response["message"]
            await self._pending.put(
                Message(
                    pgn=int(item["pgn"]),
                    source=int(item["source"]),
                    payload=base64.b64decode(item["payload"], validate=True),
                )
            )
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as exc:
            raise InternalError("invalid response from subprocess") from exc


async def connect(binary: str) -> Bus:
    try:
        process = await asyncio.create_subprocess_exec(
            binary,
            "--stdio",
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
    except OSError as exc:
        raise InternalError(f"could not start subprocess: {exc}") from exc
    return Bus(process)
