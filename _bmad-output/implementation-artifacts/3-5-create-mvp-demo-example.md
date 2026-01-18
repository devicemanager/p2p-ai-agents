# MVP Demo Example (Story 3.5)

**Status:** Completed
**Date:** 2026-01-18

## 1. Overview
The MVP Demo Example (`examples/mvp_demo.rs`) orchestrates the full interaction between two agents on a local network. It validates the core P2P AI Agents workflow: Discovery -> Task Submission -> Remote Execution -> Result Retrieval.

## 2. Implementation Details

- **File:** `examples/mvp_demo.rs`
- **Agents:** Two agents (Agent A and Agent B) are spawned within the same process but with distinct identities and peer IDs.
- **Discovery:** Uses mDNS to discover peers on the local network. Agent A polls until it finds Agent B.
- **Task Protocol:**
    - Agent A creates a `Task` struct (containing ID, prompt, priority).
    - Serializes it to JSON and sends it via `libp2p-request-response`.
    - Agent B receives the request, deserializes the `Task`, executes it using `TaskExecutor`, and returns a `TaskResult`.
    - Agent A receives the `TaskResult` and displays the output.

## 3. Verification
Run the demo using:
```bash
cargo run --example mvp_demo
```

**Expected Output:**
```text
🚀 P2P AI Agents MVP Demo
=========================

=== PHASE 1: AGENT INITIALIZATION ===
👤 Agent A initialized: 12D3Koo...
👤 Agent B initialized: 12D3Koo...

=== PHASE 2: PEER DISCOVERY ===
🔍 Searching for peers on local network (mDNS)...
✅ Discovered Agent B in 0.00s

=== PHASE 3: TASK EXCHANGE ===
📋 Created Task:
   ID:       ...
   Prompt:   "Translate 'Hello World' to French"
   Priority: 5

📤 Sending task to Agent B...
📥 Received response in 0.30s

✅ Task Completed Successfully!
   Result:      "Mock result for: 'Translate 'Hello World' to French'"
   Duration:    285ms (Execution time)
   Network RTT: 297ms

🎉 Demo complete!
```

## 4. Key Learnings & Next Steps
- **Issue:** Initial implementation of `P2PAgent` simply echoed strings.
- **Fix:** Upgraded `P2PAgent` to attempt JSON deserialization of incoming messages into `Task` objects. If successful, it invokes `TaskExecutor`.
- **Issue:** Background polling loop for Agent B was using `timeout` which caused it to drop long-running tasks or events if the timeout was too short during the `poll_once`.
- **Fix:** Removed `timeout` for the background agent's loop since it runs in its own task and blocking on `select_next_some` is desired behavior.

**Next Steps:**
- Move towards real AI model integration (Story 4.x).
- Implement more robust error handling and retries.
- Add support for streaming responses for long-running inferences.
