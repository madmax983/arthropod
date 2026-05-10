**[Havoc] Signal Deadlocks**

🧨 **The Trigger:** "Calling `.get()`, `.get_untracked()`, or `.set()` on a Signal from within an `.update()` closure on the same thread results in an RwLock deadlock."
📉 **The Stack Trace:** No panic happens, the thread hangs indefinitely due to `RwLock` re-entrance behavior on the same thread.
🧪 **Reproduction:** "Run `cargo test -p flux-state --test havoc_read_deadlock`."
😈 **Comment:** "You assumed developers would never try to read a reactive signal while updating it. You were wrong."
