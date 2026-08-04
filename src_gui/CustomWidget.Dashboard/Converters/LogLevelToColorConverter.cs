// Copyright (c) Aether Platform. Licensed under the MIT License.

using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Media;
using Windows.UI;

namespace CustomWidget.Dashboard.Converters;

/// <summary>
/// Converts a log level string to a color brush for the Diagnostics log viewer.
/// </summary>
public sealed class LogLevelToColorConverter : IValueConverter
{
    private static readonly SolidColorBrush Trace = new(Color.FromArgb(255, 97, 97, 97));    // Gray
    private static readonly SolidColorBrush Debug = new(Color.FromArgb(255, 100, 181, 246));  // Light Blue
    private static readonly SolidColorBrush Info = new(Color.FromArgb(255, 234, 234, 234));   // White
    private static readonly SolidColorBrush Warn = new(Color.FromArgb(255, 255, 179, 0));     // Amber
    private static readonly SolidColorBrush Error = new(Color.FromArgb(255, 255, 82, 82));    // Red

    public object Convert(object value, Type targetType, object parameter, string language)
    {
        string level = value?.ToString()?.ToUpperInvariant() ?? "";
        return level switch
        {
            "TRACE" => Trace,
            "DEBUG" => Debug,
            "INFO" => Info,
            "WARN" => Warn,
            "ERROR" => Error,
            _ => Info,
        };
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
        => throw new NotImplementedException();
}
