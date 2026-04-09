**Empty Slice Panics in Array Extractor Methods**
**Learning:** Functions that implicitly assume a collection is populated, like calling `.first().unwrap()` or `.last().unwrap()` on an array of `Point` data parsed from input gestures, will unconditionally panic when processing an empty sequence.
**Action:** Replace `unwrap()` calls on slice accessors with `?` to gracefully exit the function or return `None` when dealing with potentially empty data sets, and always write a test case verifying the empty input scenario.

**Testing Mutex Poisoning**
**Learning:** Testing `expect("Mutex poisoned")` panics requires deliberately poisoning a lock by panicking on a background thread that holds it, then attempting to acquire it on the main thread. However, intentionally panicking inside an inline closure on the main thread and suppressing its output via `std::panic::set_hook` is unsafe, as it alters global state and pollutes the concurrent test runner execution.
**Action:** When testing lock poisoning pathways, always spawn an isolated background thread to acquire the lock and execute the deliberate `panic!`. The main thread can then safely `join()` the thread, ignore the resulting `Err`, and proceed to trigger the targeted `expect()` panic cleanly without altering global test runner hooks.
**WGPU Readback OOM Vulnerability**
**Learning:** Calculating `total_size` and reserving memory based on input dimensions before validating against the actual readback slice length can cause catastrophic Out-Of-Memory (OOM) panics.
**Action:** When reading back textures from WGPU buffers (e.g., `unpack_readback_pixels`), always validate the source slice length against the minimum required dimensions (`padded_bytes_per_row * (height - 1) + row_len`) before allocating the destination vector to prevent Out-Of-Memory (OOM) panics from malformed dimension parameters.
