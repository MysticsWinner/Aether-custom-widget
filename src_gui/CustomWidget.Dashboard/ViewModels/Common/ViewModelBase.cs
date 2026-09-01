// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;

namespace CustomWidget.Dashboard.ViewModels.Common;

/// <summary>
/// Production-grade base class for ViewModels providing property change notification,
/// busy status management, error state tracking, and cancellation lifecycle.
/// </summary>
public abstract class ViewModelBase : INotifyPropertyChanged, IDisposable
{
    private bool _isBusy;
    private string _errorMessage = "";
    private string _title = "";
    private CancellationTokenSource? _lifecycleCts = new();
    private bool _disposed;

    public event PropertyChangedEventHandler? PropertyChanged;

    public bool IsBusy
    {
        get => _isBusy;
        set => SetProperty(ref _isBusy, value);
    }

    public string ErrorMessage
    {
        get => _errorMessage;
        set => SetProperty(ref _errorMessage, value);
    }

    public string Title
    {
        get => _title;
        set => SetProperty(ref _title, value);
    }

    public bool HasError => !string.IsNullOrWhiteSpace(_errorMessage);

    /// <summary>
    /// Gets a cancellation token bound to the lifecycle of this ViewModel.
    /// Cancelled automatically on Dispose.
    /// </summary>
    protected CancellationToken CancellationToken => _lifecycleCts?.Token ?? CancellationToken.None;

    protected bool SetProperty<T>(ref T storage, T value, [CallerMemberName] string? propertyName = null)
    {
        if (Equals(storage, value))
            return false;

        storage = value;
        OnPropertyChanged(propertyName);
        return true;
    }

    protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        if (propertyName == nameof(ErrorMessage))
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(HasError)));
        }
    }

    /// <summary>
    /// Executes an async task while managing IsBusy and ErrorMessage states safely.
    /// </summary>
    protected async Task ExecuteAsync(Func<CancellationToken, Task> action, string? actionName = null)
    {
        if (IsBusy) return;

        try
        {
            IsBusy = true;
            ErrorMessage = "";
            await action(CancellationToken).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            // Gracefully handled on cancellation
        }
        catch (Exception ex)
        {
            ErrorMessage = ex.Message;
            App.LogCrash(actionName ?? GetType().Name, ex);
        }
        finally
        {
            IsBusy = false;
        }
    }

    public virtual void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        _lifecycleCts?.Cancel();
        _lifecycleCts?.Dispose();
        _lifecycleCts = null;
    }
}
