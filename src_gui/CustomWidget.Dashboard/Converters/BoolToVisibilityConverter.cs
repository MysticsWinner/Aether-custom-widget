// Copyright (c) Aether Platform. Licensed under the MIT License.

using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Data;

namespace CustomWidget.Dashboard.Converters;

/// <summary>
/// Converts a boolean to <see cref="Visibility"/>.
/// true → Visible, false → Collapsed.
/// Pass "invert" as the ConverterParameter to reverse.
/// </summary>
public sealed class BoolToVisibilityConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, string language)
    {
        bool b = value is bool boolVal && boolVal;
        bool invert = parameter is string s && s.Equals("invert", StringComparison.OrdinalIgnoreCase);

        if (invert) b = !b;

        return b ? Visibility.Visible : Visibility.Collapsed;
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
        => value is Visibility v && v == Visibility.Visible;
}
