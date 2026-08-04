// Copyright (c) Aether Platform. Licensed under the MIT License.

using Microsoft.UI.Xaml.Data;

namespace CustomWidget.Dashboard.Converters;

/// <summary>
/// Converts byte counts to human-readable strings (KB/s, MB/s, GB/s).
/// </summary>
public sealed class BytesToHumanReadableConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, string language)
    {
        double bytes = value switch
        {
            ulong u => u,
            long l => l,
            int i => i,
            float f => f,
            double d => d,
            _ => 0,
        };

        string suffix = parameter?.ToString() ?? "/s";

        return bytes switch
        {
            >= 1_073_741_824 => $"{bytes / 1_073_741_824:F1} GB{suffix}",
            >= 1_048_576 => $"{bytes / 1_048_576:F1} MB{suffix}",
            >= 1_024 => $"{bytes / 1_024:F1} KB{suffix}",
            _ => $"{bytes:F0} B{suffix}",
        };
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
        => throw new NotImplementedException();
}
