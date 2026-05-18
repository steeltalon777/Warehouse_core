# WPF Bindings — warehouse_ffi

## Build

```bash
cargo build -p warehouse_ffi --release
```

Output: `target/release/warehouse_ffi.dll`

## P/Invoke (48 C ABI exports)

All functions follow this pattern — `CoreErrorDto` returned by value, JSON strings returned via `out IntPtr`.

```csharp
[StructLayout(LayoutKind.Sequential)]
public struct CoreErrorDto
{
    public int Code;             // 0 = success
    public IntPtr Message;       // free if Code != 0
    public IntPtr Details;       // free if Code != 0
}
```

Key P/Invoke signatures:

```csharp
public static class WarehouseNative
{
    private const string Dll = "warehouse_ffi.dll";

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void warehouse_init();

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern CoreErrorDto warehouse_open(
        string dbPath, string serverUrl, out IntPtr handle);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern CoreErrorDto warehouse_close(IntPtr handle);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern CoreErrorDto warehouse_set_user_token(
        IntPtr handle, string token);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern CoreErrorDto warehouse_bootstrap(
        IntPtr handle, out IntPtr json);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void warehouse_free_string(IntPtr ptr);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void warehouse_free_bytes(IntPtr ptr, int len);
}
```

## Memory Ownership

| Object | Returned Via | Freed By |
|---|---|---|
| `CoreErrorDto` (success) | return value, by value | Nothing — message/details are null |
| `CoreErrorDto` (error) | return value, by value | Caller frees `.Message` and `.Details` via `warehouse_free_string` |
| JSON string | `out IntPtr` | Caller via `warehouse_free_string` |
| Handle | `out IntPtr` | Caller via `warehouse_close` |
| Bytes (render) | `out IntPtr` + `out int len` | Caller via `warehouse_free_bytes` |
| `CoreErrorDto*` | by pointer (rare) | Caller via `warehouse_free_error` |

Do NOT call `warehouse_free_error` on by-value returns — only use for pointer-returning error APIs.

## Token Binding

`warehouse_set_user_token` stores the token in-memory (`FfiTokenProvider`), not in environment variables or SQLite. Token persists until `warehouse_close` or a subsequent `set_user_token` call. No token values appear in logs or error messages.

## C# Wrapper

```csharp
public class CoreHandle : IDisposable
{
    private IntPtr _handle;

    public void Open(string dbPath, string serverUrl)
    {
        WarehouseNative.warehouse_init();
        var err = WarehouseNative.warehouse_open(dbPath, serverUrl, out _handle);
        ThrowOnError(err);
    }

    public void SetUserToken(string token)
    {
        var err = WarehouseNative.warehouse_set_user_token(_handle, token);
        ThrowOnError(err);
    }

    public string Bootstrap()
    {
        var err = WarehouseNative.warehouse_bootstrap(_handle, out var json);
        ThrowOnError(err);
        var result = Marshal.PtrToStringAnsi(json);
        WarehouseNative.warehouse_free_string(json);
        return result;
    }

    public void Dispose()
    {
        if (_handle != IntPtr.Zero)
        {
            WarehouseNative.warehouse_close(_handle);
            _handle = IntPtr.Zero;
        }
    }

    private static void ThrowOnError(CoreErrorDto err)
    {
        if (err.Code == 0) return;
        var msg = Marshal.PtrToStringAnsi(err.Message) ?? "unknown error";
        WarehouseNative.warehouse_free_string(err.Message);
        if (err.Details != IntPtr.Zero)
            WarehouseNative.warehouse_free_string(err.Details);
        throw new InvalidOperationException($"Core error {err.Code}: {msg}");
    }
}
```

## DI Registration

```csharp
services.AddSingleton<CoreHandle>(sp => {
    var handle = new CoreHandle();
    handle.Open("warehouse.db", Configuration["SyncServer:Url"]);
    return handle;
});
```
