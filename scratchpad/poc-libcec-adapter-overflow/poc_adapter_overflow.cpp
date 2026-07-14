// SPDX-License-Identifier: GPL-2.0-or-later
//
// Proof-of-concept: transmit-path buffer overflows in libCEC platform adapters.
//
// Upstream commit 646ed0a ("fixed: bound RPi transmit payload to the CEC frame
// size (#602)") fixed the same class of bug in the RPi adapter. The four
// adapters exercised here still copy command.parameters (up to
// CEC_MAX_DATA_PACKET_SIZE = 64 bytes, see cectypes.h) into a small fixed
// stack buffer with either no bound or an off-by-one bound. Verified against
// upstream master 9396b7d (2026-07-14).
//
// Each *_fill() below is the buffer-fill logic copied VERBATIM from the named
// adapter's CxxxCECAdapterCommunication::Write(), reduced to the part that
// touches the stack buffer. The real overflow happens in this code, before the
// write()/ioctl() syscall, so no device or libCEC linkage is needed.
//
// Build & run:  ./run_poc.sh   (compiles with -fsanitize=address)
//
// Usage:        ./poc <exynos|aocec|tegra|linux> <nparams>
//   exit 0  -> completed without a detected overflow
//   nonzero -> AddressSanitizer aborted on the overflow

#include <cectypes.h>

#include <linux/cec.h>      // real struct cec_msg (msg[CEC_MAX_MSG_SIZE] == 16)

#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <cstring>

using namespace CEC;

// Build a command carrying `nparams` operands (clamped to the datapacket max).
static cec_command make_command(int nparams)
{
  cec_command cmd;
  cec_command::Format(cmd, CECDEVICE_PLAYBACKDEVICE1, CECDEVICE_TV,
                      CEC_OPCODE_SET_OSD_STRING);
  for (int i = 0; i < nparams; i++)
    cmd.parameters.PushBack((uint8_t)(0xA0 | (i & 0x0f)));
  return cmd;
}

// ---------------------------------------------------------------------------
// Exynos  — adapter/Exynos/ExynosCECAdapterCommunication.cpp:108 Write()
// Guard at :118 omits the leading header byte (buffer[0]), so a 15-operand
// command writes buffer[16] on a 16-byte buffer. Off-by-one, 1-byte OOB.
// ---------------------------------------------------------------------------
static void exynos_fill(const cec_command &data)
{
  const size_t CEC_MAX_FRAME_SIZE = 16;
  uint8_t buffer[CEC_MAX_FRAME_SIZE];
  int32_t size = 1;

  if ((size_t)data.parameters.size + data.opcode_set > sizeof(buffer))
    return;                                          // guard is too weak

  buffer[0] = (data.initiator << 4) | (data.destination & 0x0f);
  if (data.opcode_set)
  {
    buffer[1] = data.opcode;
    size++;
    memcpy(&buffer[size], data.parameters.data, data.parameters.size);
    size += data.parameters.size;
  }
  volatile uint8_t sink = buffer[0]; (void)sink; (void)size;
}

// ---------------------------------------------------------------------------
// AOCEC  — adapter/AOCEC/AOCECAdapterCommunication.cpp:123 Write()
// Identical guard/omission to Exynos (:135). Off-by-one, 1-byte OOB.
// ---------------------------------------------------------------------------
static void aocec_fill(const cec_command &data)
{
  const size_t CEC_MAX_FRAME_SIZE = 16;
  uint8_t buffer[CEC_MAX_FRAME_SIZE];
  int32_t size = 1;

  if ((size_t)data.parameters.size + data.opcode_set > sizeof(buffer))
    return;                                          // guard is too weak

  buffer[0] = (data.initiator << 4) | (data.destination & 0x0f);
  if (data.opcode_set)
  {
    buffer[1] = data.opcode;
    size++;
    memcpy(&buffer[size], data.parameters.data, data.parameters.size);
    size += data.parameters.size;
  }
  volatile uint8_t sink = buffer[0]; (void)sink; (void)size;
}

// ---------------------------------------------------------------------------
// Tegra  — adapter/Tegra/TegraCECAdapterCommunication.cpp:122 Write()
// Guard at :146 runs AFTER the write and uses '>' not '>=', so cmdData[16]
// (on a 16-byte buffer) is written before the overflow is detected.
// ---------------------------------------------------------------------------
static void tegra_fill(const cec_command &data)
{
  const int TEGRA_CEC_FRAME_MAX_LENGTH = 16;
  int size = 0;
  unsigned char cmdData[TEGRA_CEC_FRAME_MAX_LENGTH];
  unsigned char addr = (data.initiator << 4) | (data.destination & 0x0f);

  cmdData[size] = addr;
  size++;
  if (data.opcode_set) {
    cmdData[size] = data.opcode;
    size++;
  }
  for (int i = 0; i < data.parameters.size; i++) {
    cmdData[size] = data.parameters.data[i];        // OOB write when size==16
    size++;
    if (size > TEGRA_CEC_FRAME_MAX_LENGTH)           // detected one byte late
      return;
  }
  volatile unsigned char sink = cmdData[0]; (void)sink; (void)size;
}

// ---------------------------------------------------------------------------
// Linux  — adapter/Linux/LinuxCECAdapterCommunication.cpp:204 Write()
// No bound at all (:217): memcpy of up to 64 operands into the kernel
// struct cec_msg's msg[CEC_MAX_MSG_SIZE] (== 16). Unlike the other adapters
// the destination is a field *embedded in a larger struct* (56 bytes here), so
// the write first corrupts sibling members (reply, tx_status, rx_status, ...)
// and only escapes the whole object beyond ~24 operands. That intra-struct
// corruption is invisible to ASan, so we detect it deterministically: after
// the copy, scan the bytes past msg[15] for the operand fill pattern.
// Returns true iff the copy wrote past the 16-byte CEC message field.
// ---------------------------------------------------------------------------
static bool linux_fill(const cec_command &data)
{
  struct cec_msg msg;
  cec_msg_init(&msg, data.initiator, data.destination);   // zero-inits the tail
  if (data.opcode_set)
  {
    msg.msg[msg.len++] = data.opcode;
    if (data.parameters.size)
    {
      memcpy(&msg.msg[msg.len], data.parameters.data, data.parameters.size);
      msg.len += data.parameters.size;
    }
  }

  // Bytes past the semantic 16-byte field, up to the end of the struct.
  const uint8_t *tail = (const uint8_t *)&msg + offsetof(struct cec_msg, msg)
                        + CEC_MAX_MSG_SIZE;
  const uint8_t *end  = (const uint8_t *)&msg + sizeof(struct cec_msg);
  for (const uint8_t *p = tail; p < end; p++)
    if ((*p & 0xf0) == 0xa0)          // operand fill pattern (0xA0 | ...)
      return true;                    // wrote into sibling struct fields
  return msg.len > CEC_MAX_MSG_SIZE;   // or staged an out-of-range length
}

int main(int argc, char **argv)
{
  if (argc < 3) {
    fprintf(stderr, "usage: %s <exynos|aocec|tegra|linux> <nparams>\n", argv[0]);
    return 2;
  }
  const char *which = argv[1];
  int nparams       = atoi(argv[2]);
  cec_command cmd   = make_command(nparams);

  printf("case=%-7s nparams=%2d parameters.size=%2u ... ",
         which, nparams, cmd.parameters.size);
  fflush(stdout);

  // exynos/aocec/tegra write standalone stack arrays: AddressSanitizer aborts
  // the process on the out-of-bounds write, so reaching the end here means no
  // overflow. linux writes into an embedded struct field, so it reports the
  // overflow via return value (ASan can't see the intra-struct corruption).
  if      (!strcmp(which, "exynos")) exynos_fill(cmd);
  else if (!strcmp(which, "aocec"))  aocec_fill(cmd);
  else if (!strcmp(which, "tegra"))  tegra_fill(cmd);
  else if (!strcmp(which, "linux")) {
    if (linux_fill(cmd)) { printf("OVERFLOW: wrote past msg[16]\n"); return 1; }
  }
  else { fprintf(stderr, "unknown case '%s'\n", which); return 2; }

  printf("no overflow detected\n");
  return 0;
}
