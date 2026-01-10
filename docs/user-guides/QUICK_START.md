# P2P AI Agents - Quick Start Guide (MVP)

**Version**: MVP 0.1.0  
**Last Updated**: 2026-01-31  
**Estimated Time**: 5 minutes

---

## Prerequisites

- **Rust**: 1.75.0+ ([install](https://rustup.rs/))
- **OS**: macOS or Linux
- **Network**: Local network (mDNS)

---

## Run MVP Demo

```bash
git clone https://github.com/p2p-ai-agents/p2p-ai-agents.git
cd p2p-ai-agents
cargo run --release --example mvp_demo
```

**Expected output** (~3 seconds):
```
🚀 P2P AI Agents - MVP Demo
✅ Agent A started
✅ Agent B started
✅ Discovered (2.1s)
📤 Task: "Hello, P2P AI!"
📥 Result: "Processed: Hello, P2P AI!" (18ms)
✅ Demo complete!
```

---

## What Just Happened?

1. **Discovery**: 2 agents found each other via mDNS
2. **Task**: Agent A sent task to Agent B
3. **Execution**: Agent B ran mock inference
4. **Result**: Agent A received result

**Code**: 1,030 LOC Rust  
**Dependencies**: 7 crates (tokio, libp2p, ed25519, serde)  
**Performance**: 2.1s discovery, 18ms task latency

---

## Troubleshooting

**Build fails**: `rustup update` (need Rust 1.75+)  
**Discovery timeout**: Check firewall (UDP 5353)  
**Port in use**: `pkill -f mvp_demo`

---

## MVP Limitations

- ❌ Local network only (no NAT traversal)
- ❌ Mock inference (no real models)
- ❌ Ephemeral keys (not saved)

**Post-MVP**: Internet-wide P2P, real models, persistence

---

## Next Steps

- **Code**: `cat examples/mvp_demo.rs`
- **Tests**: `cargo test --all`
- **Architecture**: `docs/architecture/mvp-architecture.md`
- **Issues**: https://github.com/p2p-ai-agents/issues

---

**Status**: ✅ Story 4.1 Complete  
**Demo video**: Coming Feb 1
