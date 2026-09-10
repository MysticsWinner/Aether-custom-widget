// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.Generic;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Models;

namespace CustomWidget.Dashboard.Services.Interfaces;

/// <summary>
/// Service abstraction for loading, saving, and managing per-widget layout and display configurations.
/// </summary>
public interface IWidgetSettingsService
{
    Task<Dictionary<string, WidgetDisplayOptions>> LoadAllSettingsAsync();
    Task<WidgetDisplayOptions> GetSettingsAsync(string widgetId);
    WidgetDisplayOptions Load(string widgetId);
    Task SaveSettingsAsync(string widgetId, WidgetDisplayOptions options);
    Task SetPositionAsync(string widgetId, int x, int y);
    Task SetLockedAsync(string widgetId, bool locked);
    Task ToggleLockAsync(string widgetId);
    Task SetOpacityAsync(string widgetId, double opacity);
    Task SetScaleAsync(string widgetId, double scale);
    Task SetEnabledAsync(string widgetId, bool enabled);
    Task ResetAsync(string widgetId);
}
