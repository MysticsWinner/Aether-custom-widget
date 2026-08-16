// Copyright (c) Aether Platform. Licensed under the MIT License.

using Microsoft.UI.Xaml.Data;

namespace CustomWidget.Dashboard.Converters;

public sealed class BoolToTextConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, string language)
    {
        if (value is bool boolVal)
        {
            return boolVal ? "Unload Widget" : "Load Widget";
        }
        return "Load Widget";
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
    {
        throw new NotImplementedException();
    }
}
