# Low-Level Plugin C API & ABI Specification

**C ABI Foreign Function Interface (FFI) for Out-of-Process Plugins**

---

## 1. Native C Function Signatures

Out-of-process DLL plugins export standard C symbols:

```c
#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint64_t timestamp_ms;
    float cpu_usage_pct;
    float gpu_usage_pct;
    float memory_used_mb;
    float memory_total_mb;
} AetherTelemetryData;

__declspec(dllexport) int32_t aether_plugin_init(void);
__declspec(dllexport) int32_t aether_plugin_tick(const AetherTelemetryData* data);
__declspec(dllexport) int32_t aether_plugin_shutdown(void);

#ifdef __cplusplus
}
#endif
```

Return codes: `0` for success (`AETHER_OK`), negative values for error codes.
