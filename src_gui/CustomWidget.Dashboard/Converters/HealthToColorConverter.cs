// Copyright (c) Aether Platform. Licensed under the MIT License.

using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Media;
using Windows.UI;

namespace CustomWidget.Dashboard.Converters;

/// <summary>
/// Converts a health status string to a color brush.
/// "Healthy" → Green, "Degraded" → Amber, "Failed" → Red.
/// </summary>
public sealed class HealthToColorConverter : IValueConverter
{
    private static readonly SolidColorBrush Green = new(Color.FromArgb(255, 0, 230, 118));
    private static readonly SolidColorBrush Amber = new(Color.FromArgb(255, 255, 179, 0));
    private static readonly SolidColorBrush Red = new(Color.FromArgb(255, 255, 82, 82));
    private static readonly SolidColorBrush Gray = new(Color.FromArgb(255, 158, 158, 158));

    public object Convert(object value, Type targetType, object parameter, string language)
    {
        string health = value?.ToString() ?? "";
        return health switch
        {
            "Healthy" => Green,
            "Degraded" => Amber,
            "Failed" => Red,
            _ => Gray,
        };
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
        => throw new NotImplementedException();
}
