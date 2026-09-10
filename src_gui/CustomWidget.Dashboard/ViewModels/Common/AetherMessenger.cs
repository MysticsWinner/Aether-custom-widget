// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels.Common;

/// <summary>
/// Thread-safe in-memory message aggregator for zero-coupling cross-ViewModel communication.
/// </summary>
public sealed class AetherMessenger
{
    private static readonly Lazy<AetherMessenger> _default = new(() => new AetherMessenger());
    public static AetherMessenger Default => _default.Value;

    private readonly ConcurrentDictionary<Type, List<Delegate>> _subscribers = new();

    /// <summary>
    /// Subscribes an action to messages of type <typeparamref name="TMessage"/>.
    /// </summary>
    public void Subscribe<TMessage>(Action<TMessage> action)
    {
        var type = typeof(TMessage);
        _subscribers.AddOrUpdate(type,
            _ => new List<Delegate> { action },
            (_, list) =>
            {
                lock (list)
                {
                    list.Add(action);
                }
                return list;
            });
    }

    /// <summary>
    /// Unsubscribes an action from messages of type <typeparamref name="TMessage"/>.
    /// </summary>
    public void Unsubscribe<TMessage>(Action<TMessage> action)
    {
        var type = typeof(TMessage);
        if (_subscribers.TryGetValue(type, out var list))
        {
            lock (list)
            {
                list.Remove(action);
            }
        }
    }

    /// <summary>
    /// Publishes a message to all active subscribers of type <typeparamref name="TMessage"/>.
    /// </summary>
    public void Send<TMessage>(TMessage message)
    {
        var type = typeof(TMessage);
        if (_subscribers.TryGetValue(type, out var list))
        {
            Delegate[] targets;
            lock (list)
            {
                targets = list.ToArray();
            }

            foreach (var target in targets)
            {
                if (target is Action<TMessage> typedAction)
                {
                    try
                    {
                        typedAction(message);
                    }
                    catch (Exception ex)
                    {
                        DashboardLogger.Error("AetherMessenger", $"Error invoking subscriber for message {type.Name}: {ex.Message}", ex);
                    }
                }
            }
        }
    }

    /// <summary>
    /// Clears all active message subscriptions.
    /// </summary>
    public void Clear()
    {
        _subscribers.Clear();
    }
}

// ── Standard Cross-ViewModel Messages ──

public record WidgetRegistryChangedMessage(string WidgetId, bool IsLoaded);
public record SnapshotRestoredMessage(string SnapshotId);
public record ThemeModeChangedMessage(string Mode);
public record EngineConnectionStateMessage(bool IsConnected, string EngineVersion);
public record DesktopOverlayToggledMessage(bool IsVisible);
