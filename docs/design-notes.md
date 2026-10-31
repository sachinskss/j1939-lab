# Design notes

## RTS/CTS transport

The demonstration implements BAM only. RTS/CTS needs two cooperating peers and a
flow-control state machine. The virtual bus in this project can deliver a control
frame, but a one-peer demonstration cannot prove that sender and receiver agree
on connection limits, window size, retry behavior, and abort handling.

The next design step is a `ConnectionKey` containing source, destination, and
PGN, with explicit sender and receiver states. The sender waits for CTS before
emitting each permitted window. The receiver limits each window and sends CTS,
EndOfMessageAck, or Abort. Tests should inject lost CTS, zero-length CTS, invalid
window sizes, and timeout at each state. Hardware support is a later shell
backend; it does not belong in `j1939-tp`.

The RTS control frame is not accepted by the current receiver. The transition
from Idle to Session on RTS is intentionally unimplemented and untested because
RTS/CTS is outside the v0.1.0 scope. In an interview, describe this as the peer
and flow-control cut, then point to the state-machine design above.

## Address claiming

The core defines the address-claimed PGN and the `Name` fields. This project does
not implement arbitration or address selection. A complete implementation needs
to compare 64-bit NAME values, handle address conflicts, support cannot-claim,
and retry with a legal address. Those transitions need at least two simulated
nodes and a network-management policy.

The next design step is a pure state machine that accepts received claim frames
and caller-supplied elapsed time. A shell owns node discovery and frame I/O. The
test plan should cover a free address, a conflict won by the lower NAME, a losing
node moving to another address, and a node that cannot claim. The current `Name`
type is a data model only.

## Recorded scope cuts

- PyO3 is deferred. The Python SDK uses a versioned JSON subprocess protocol.
- Windows and macOS CI are deferred. CI targets Linux.
- DBC parsing and ISOBUS VT, TC, and object pools are out of scope.
- The clock boundary uses caller-supplied monotonic milliseconds because the
  Rust core is `no_std` and cannot depend on `std::time::Instant`.
