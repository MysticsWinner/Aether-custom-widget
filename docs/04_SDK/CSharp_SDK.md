# C# .NET 8 Widget SDK Reference

**Purpose**: Guide for writing Aether widgets and desktop tools using C# .NET 8 and WinUI 3.  
**Audience**: .NET Developers.  
**Prerequisites**: [Widget_SDK.md](Widget_SDK.md).  
**Related Documents**: [Dashboard.md](../05_GUI/Dashboard.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / SDK Guide  
**Owner**: .NET SDK Team  

---

## 1. Win32 Named Pipe IPC Client

C# applications connect to `\\.\pipe\CustomWidgetEngineControlPipe` using `NamedPipeClientStream` and exchange JSON payloads matching `ControlCommand` and `MetricPayload`.

---

## Future Work
- Publish `Aether.Sdk` NuGet package.

## Known Issues
- None.

## References
- [src_gui/CustomWidget.Dashboard/](file:///d:/Code/Aether-custom-widget/src_gui/CustomWidget.Dashboard)

## Related Documents
- [Widget_SDK.md](Widget_SDK.md)
