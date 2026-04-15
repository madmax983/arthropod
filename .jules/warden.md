**2025-02-23 - Handle null pointer dereferences and integer overflow**
**Threat:** Null handles (like `HWND(0)`) could be passed to unsafe FFI functions in `plat-core`, and large texture readback size calculations in `render-engine` could cause integer overflow, bypassing limits and resulting in unbounded allocations (DoS/OOM).
**Defense:** Added explicit `is_null()` validation to Windows FFI handle parameters, and updated `render-engine` allocation math to use `checked_mul` so large dimensions safely error instead of overflowing.

**2025-02-23 - Validate RegQueryValueExW output variables**
**Threat:** Missing validation of `reg_type` and `data_size` output variables when calling `RegQueryValueExW` via FFI could allow malicious or incorrect registry keys to trigger out-of-bounds reads or logic bugs by misinterpreting raw byte buffers as the wrong data type.
**Defense:** Updated the `is_ok()` success checks to strictly ensure `reg_type` exactly matches the expected registry type (`REG_DWORD` or `REG_SZ`) and `data_size` exactly matches the expected size in memory before processing.

**2025-02-23 - Handle null pointer dereference in WM_NCCREATE message**
**Threat:** Null pointers (like `lparam.0 == 0`) could be passed to unsafe FFI functions in `wndproc` in `plat-core` during window creation `WM_NCCREATE`, and dereferencing it as `*const CREATESTRUCTW` causes a null pointer dereference leading to undefined behavior or a crash (DoS).
**Defense:** Added explicit `== 0` validation to Windows `lparam.0` when handling `WM_NCCREATE` to safely return an error (`LRESULT(0)`) instead of blindly dereferencing the pointer.

**2025-02-23 - Handle null HWND in DirectComposition target creation**
**Threat:** A null window handle (`HWND(0)`) could be passed to the unsafe COM API `CreateTargetForHwnd` in `plat-core` (`create_target_for_hwnd`), causing undefined behavior or crashes (DoS) when attempting to access the underlying window surface.
**Defense:** Added an explicit `is_null()` validation to the `HWND` parameter before entering the `unsafe` FFI block, returning a safe `Error::from(E_HANDLE)` to gracefully handle the failure.

**2025-02-23 - Handle infinite loop DoS in cyclic parent structures**
**Threat:** A cyclic scene graph structure could cause an infinite loop in `arthropod-ecs` parent traversal, leading to CPU exhaustion (Denial of Service).
**Defense:** Added a `HashSet::insert` check during traversal to detect duplicate parent nodes, gracefully breaking out of the loop and preventing the application from freezing.
**2025-02-23 - Handle invalid HWND access violations via IsWindow**
**Threat:** A null or invalid window handle (`HWND`) could be passed to unsafe FFI functions and COM APIs in `plat-core` (`create_target_for_hwnd` and `get_window_id`), causing undefined behavior or access violations (DoS) when attempting to read the window data. Checking `.is_null()` is insufficient to verify the handle refers to a valid Windows object.
**Defense:** Replaced the `.is_null()` check with the system's `IsWindow` API before safely accessing user data or passing it to COM targets.
