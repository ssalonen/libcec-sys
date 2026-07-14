# libCEC adapter transmit-path buffer overflows (Exynos, AOCEC, Tegra, Linux)

Standalone proof-of-concept for a family of transmit-buffer overflows in the
libCEC platform adapters. These are the **same class of bug** as the RPi issue
[#602](https://github.com/Pulse-Eight/libcec/issues/602), fixed for RPi only by
commit `646ed0a` ("fixed: bound RPi transmit payload to the CEC frame size").
The four adapters below are still unbounded / off-by-one on **upstream master
`9396b7d`** (2026-07-14).

## Root cause

A `cec_command`'s `parameters` (`cec_datapacket`) holds up to
`CEC_MAX_DATA_PACKET_SIZE = 64` operand bytes (`include/cectypes.h`). Each
adapter's `Write()` copies `command.opcode` + those operands into a fixed
16-byte on-stack frame laid out as `[header][opcode][operands...]`, so the last
operand that fits is #14 and operand #15 lands one byte past the buffer.

| Adapter | Write() (master `9396b7d`) | Destination | Bug |
|---|---|---|---|
| Exynos | `Exynos/ExynosCECAdapterCommunication.cpp:108` | `buffer[16]` | guard at `:118` omits the header byte → 1-byte OOB |
| AOCEC  | `AOCEC/AOCECAdapterCommunication.cpp:123`   | `buffer[16]` | identical omission at `:135` → 1-byte OOB |
| Tegra  | `Tegra/TegraCECAdapterCommunication.cpp:122` | `cmdData[16]` | bound check `:146` runs *after* the write and uses `>` not `>=` |
| Linux  | `Linux/LinuxCECAdapterCommunication.cpp:204` | `struct cec_msg.msg[16]` | no bound at all at `:217` |

For reference, the adapters that get it right and can be used as the fix
template: `IMX/IMXCECAdapterCommunication.cpp:119`
(`parameters.size + opcode_set + 1 > sizeof(message)`) and
`TDA995x/TDA995xCECAdapterCommunication.cpp`.

## Reachability

`cec_datapacket::PushBack` accepts up to 64 bytes, so any caller that builds an
over-long command (e.g. `libcec_transmit` with a large `cec_command`, or a
malformed/oversized frame reconstructed into a command) reaches the copy.
Real CEC traffic is ≤16 bytes, which is exactly why the missing check went
unnoticed.

## What the PoC does

`poc_adapter_overflow.cpp` contains each adapter's buffer-fill logic copied
**verbatim** from `Write()` (only trimmed to the part that touches the stack
buffer — no device or libCEC linkage needed, because the overflow happens
before the `write()`/`ioctl()` syscall). It builds a `cec_command` with N
operands using the real `cec_command` / `cec_datapacket` types and runs the
copy.

- **Exynos / AOCEC / Tegra** write standalone stack arrays, so AddressSanitizer
  aborts on the out-of-bounds write.
- **Linux** writes into a field embedded in the 56-byte kernel `struct cec_msg`
  (`msg[]` at offset 32). The overflow first corrupts sibling members
  (`reply`, `tx_status`, `rx_status`, …) and only escapes the whole object past
  ~24 operands — where glibc `_FORTIFY_SOURCE` aborts. The intra-struct part is
  invisible to ASan, so the PoC detects it deterministically by scanning the
  bytes past `msg[15]` for the operand fill pattern.

## Run

```
./run_poc.sh [path-to-libcec-checkout]     # default: ../../vendor
```

Requires `g++` with AddressSanitizer. For each adapter it runs the last safe
operand count (expected clean) and the first overflowing count (expected to be
caught), and prints a PASS/FAIL matrix. Expected output:

```
ADAPTER  SAFE (expect clean)    OVERFLOW (expect ASan)
exynos   n=14 PASS             n=15 PASS
aocec    n=14 PASS             n=15 PASS
tegra    n=14 PASS             n=15 PASS
linux    n=14 PASS             n=15 PASS
RESULT: all cases behaved as predicted ...
```

Direct invocation: `./.build/poc <exynos|aocec|tegra|linux> <nparams>`.

## Suggested fix

Mirror the IMX guard at the top of each `Write()`, rejecting the command before
the copy (accounting for header + opcode + operands):

```cpp
if ((size_t)data.parameters.size + data.opcode_set + 1 > sizeof(buffer)) {
  LIB_CEC->AddLog(CEC_LOG_ERROR, "%s: command too large", __func__);
  return ADAPTER_MESSAGE_STATE_ERROR;
}
```

For Tegra, additionally move the length check *before* the write and use `>=`.
For Linux, bound against `CEC_MAX_MSG_SIZE` before the `memcpy`.
