// Copyright (c) Aether Platform. Licensed under the MIT License.

using Microsoft.UI;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Media;
using Windows.UI;

namespace CustomWidget.Dashboard.Converters;

/// <summary>
/// Converts a percentage value (0–100) to a color:
/// 0–60 → Green, 60–85 → Amber, 85–100 → Red.
/// </summary>
public sealed class PercentToColorConverter : IValueConverter
{
    private static readonly SolidColorBrush Green = new(Color.FromArgb(255, 0, 230, 118));   // #00E676
    private static readonly SolidColorBrush Amber = new(Color.FromArgb(255, 255, 179, 0));   // #FFB300
    private static readonly SolidColorBrush Red = new(Color.FromArgb(255, 255, 82, 82));     // #FF5252

    public object Convert(object value, Type targetType, object parameter, string language)
    {
        float pct = value switch
        {
            float f => f,
            double d => (float)d,
            int i => i,
            _ => 0f,
        };

        return pct switch
        {
            >= 85f => Red,
            >= 60f => Amber,
            _ => Green,
        };
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
        => throw new NotImplementedException();
}
