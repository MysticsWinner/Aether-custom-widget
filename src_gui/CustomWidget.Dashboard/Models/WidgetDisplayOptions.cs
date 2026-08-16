// Copyright (c) Aether Platform. Licensed under the MIT License.

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// Display options for a single desktop widget, mirroring the Rust WidgetConfig schema.
/// Persisted per-widget under %LOCALAPPDATA%\Aether\widget_settings\&lt;widget_id&gt;.json
/// and synchronised with the Core Engine via IPC UpdateWidgetDisplayOptions commands.
/// </summary>
public class WidgetDisplayOptions
{
    public string WidgetId { get; set; } = string.Empty;
    public double Opacity { get; set; } = 1.0;
    public double Scale { get; set; } = 1.0;
    public bool Locked { get; set; } = false;
    public bool Enabled { get; set; } = true;
    public bool QuickSwap { get; set; } = false;
}
