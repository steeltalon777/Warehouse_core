# Android Bindings — warehouse_ffi

## Build Targets

```bash
rustup target add aarch64-linux-android x86_64-linux-android
cargo build -p warehouse_ffi --target aarch64-linux-android --release
cargo build -p warehouse_ffi --target x86_64-linux-android --release
```

Output: `target/<target>/release/libwarehouse_ffi.so`

## Calling Convention

`warehouse_ffi` exports **C ABI** (`extern "C"`) functions — not JNI. Use JNA or a manual JNI bridge layer to call from Kotlin.

## C ABI Function Index (48 exports)

| Function | Signature | Notes |
|---|---|---|
| `warehouse_init` | `void()` | Call once before first `open` |
| `warehouse_open` | `ErrorCode(db,serverUrl) -> Handle` | Returns handle via `out` ptr |
| `warehouse_close` | `ErrorCode(Handle)` | Frees handle |
| `warehouse_set_user_token` | `ErrorCode(Handle, CString)` | Stores token in-memory (see below) |
| `warehouse_set_device_token` | `ErrorCode(Handle, CString)` | Stores token in-memory (see below) |
| `warehouse_refresh_identity` | `ErrorCode(Handle)` | GET /auth/context |
| `warehouse_bootstrap` | `ErrorCode(Handle) -> JsonStr` | Bootstrap result as JSON |
| `warehouse_pull_once` | `ErrorCode(Handle) -> JsonStr` | SyncRunSummary as JSON |
| `warehouse_search_catalog` | `ErrorCode(Handle, CString) -> JsonStr` | Results as JSON array |
| `warehouse_list_balances` | `ErrorCode(Handle, i32) -> JsonStr` | By site ID |
| `warehouse_list_operations` | `ErrorCode(Handle, i32, u32, u32) -> JsonStr` | site, page, page_size |
| `warehouse_free_string` | `void(CString*)` | Free JSON result |
| `warehouse_free_error` | `void(ErrorCode*)` | Free error by pointer |
| `warehouse_free_bytes` | `void(u8*, usize)` | Free render result |
| *(plus 35 more draft/outbox/sync/temp/docs/conflict functions)* | | |

## Token Binding

`warehouse_set_user_token` stores the token in the `FfiTokenProvider` (in-memory, not in env vars). Tokes persist until `warehouse_close` or a new `set_user_token` call. No token values are written to SQLite or logs.

## Error Envelope (returned by value)

```c
typedef struct {
    int32_t code;        // 0 = success, >0 = error (see CORE_FACADE_V1.md for codes)
    char* message;       // must free with warehouse_free_string if code != 0
    char* details;       // must free with warehouse_free_string if code != 0
} CoreErrorDto;
```

Most functions return `CoreErrorDto` **by value** — callers must read `code` and if non-zero, free `message` and `details` via `warehouse_free_string`. Do NOT call `warehouse_free_error` on by-value returns.

`warehouse_free_error(ErrorDto*)` is only for the (rare) case where an API returns `ErrorDto*` by pointer.

## JNA Example

```kotlin
interface WarehouseFfi : com.sun.jna.Library {
    fun warehouse_init()
    fun warehouse_open(dbPath: String, serverUrl: String, handle: PointerByReference): CoreErrorDto.ByValue
    fun warehouse_close(handle: Pointer): CoreErrorDto.ByValue
    fun warehouse_set_user_token(handle: Pointer, token: String): CoreErrorDto.ByValue
    fun warehouse_refresh_identity(handle: Pointer): CoreErrorDto.ByValue
    fun warehouse_bootstrap(handle: Pointer, json: PointerByReference): CoreErrorDto.ByValue
    fun warehouse_free_string(ptr: Pointer)
    fun warehouse_free_error(dto: Pointer)
}

@Structure.FieldOrder("code", "message", "details")
class CoreErrorDto : Structure() {
    @JvmField var code: Int = 0
    @JvmField var message: Pointer? = null
    @JvmField var details: Pointer? = null

    class ByValue : CoreErrorDto(), Structure.ByValue
}
```

## Gradle + JNA

```kotlin
dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
}

android {
    defaultConfig {
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }
}
```

## Hilt DI Module

```kotlin
@Module @InstallIn(SingletonComponent::class)
object CoreModule {
    @Provides @Singleton
    fun provideCoreHandle(): CoreHandle {
        // Swap to RustCoreHandle<JNA> when .so is available
        return SurrogateCoreHandle()
    }
}
```

## Surrogate → Rust Swap

1. Build native `.so` for both ABIs
2. Place in `src/main/jniLibs/arm64-v8a/` and `.../x86_64/`
3. Change DI module: `SurrogateCoreHandle()` → `RustCoreHandle(JNA)`
4. ViewModels and screens unchanged — they depend on `CoreHandle` interface
