// WorkerW Desktop Layer Hook Implementation
#if defined(_WIN32) || defined(__CYGWIN__)
#include <windows.h>

extern "C" __declspec(dllexport) HWND FetchDesktopWorkerWWindow() {
    HWND progman = FindWindowW(L"Progman", NULL);
    if (!progman) return NULL;

    // Send 0x052C message to Progman to spawn WorkerW behind desktop icons
    SendMessageTimeoutW(progman, 0x052C, 0, 0, SMTO_NORMAL, 1000, NULL);

    HWND workerw = NULL;
    EnumWindows([](HWND top_window, LPARAM lparam) -> BOOL {
        HWND shell_dll = FindWindowExW(top_window, NULL, L"SHELLDLL_DefView", NULL);
        if (shell_dll != NULL) {
            *(HWND*)lparam = FindWindowExW(NULL, top_window, L"WorkerW", NULL);
            return FALSE;
        }
        return TRUE;
    }, (LPARAM)&workerw);

    return workerw;
}
#else
extern "C" void* FetchDesktopWorkerWWindow() {
    return nullptr;
}
#endif
