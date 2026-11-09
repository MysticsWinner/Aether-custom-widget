// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.Generic;
using CommunityToolkit.Mvvm.ComponentModel;

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// 8-point resize handles for the 2D Visual Canvas Layout Editor (Phase 23.3).
/// </summary>
public enum ResizeHandle
{
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left
}

/// <summary>
/// Represents an interactive widget bounding box on the 2D Visual Desktop Canvas Editor,
/// supporting magnetic grid snapping, 8-point resize handles, and alignment guide calculation.
/// </summary>
public partial class CanvasWidgetBox : ObservableObject
{
    [ObservableProperty] private string _widgetId = "";
    [ObservableProperty] private string _displayName = "";
    [ObservableProperty] private int _x = 100;
    [ObservableProperty] private int _y = 100;
    [ObservableProperty] private int _width = 320;
    [ObservableProperty] private int _height = 200;
    [ObservableProperty] private bool _isLocked;

    public string CoordinatesSummary => $"({X}, {Y}) — {Width}×{Height} px";

    partial void OnXChanged(int value) => OnPropertyChanged(nameof(CoordinatesSummary));
    partial void OnYChanged(int value) => OnPropertyChanged(nameof(CoordinatesSummary));
    partial void OnWidthChanged(int value) => OnPropertyChanged(nameof(CoordinatesSummary));
    partial void OnHeightChanged(int value) => OnPropertyChanged(nameof(CoordinatesSummary));

    /// <summary>
    /// Snaps a 2D desktop coordinate pair (x, y) to the nearest magnetic grid multiple.
    /// </summary>
    public static (int SnappedX, int SnappedY) SnapToGrid(int x, int y, int gridSize)
    {
        int step = Math.Max(1, gridSize);
        int sx = (int)Math.Round((double)x / step) * step;
        int sy = (int)Math.Round((double)y / step) * step;
        return (Math.Max(0, sx), Math.Max(0, sy));
    }

    /// <summary>
    /// Applies an 8-point resize handle drag delta while enforcing minimum widget dimensions and desktop bounds.
    /// </summary>
    public void ApplyResize(
        ResizeHandle handle,
        int deltaX,
        int deltaY,
        int minWidth = 120,
        int minHeight = 80,
        int desktopWidth = 1920,
        int desktopHeight = 1080)
    {
        if (IsLocked) return;

        int newX = X;
        int newY = Y;
        int newW = Width;
        int newH = Height;

        switch (handle)
        {
            case ResizeHandle.Right:
                newW = Math.Clamp(Width + deltaX, minWidth, desktopWidth - X);
                break;
            case ResizeHandle.Bottom:
                newH = Math.Clamp(Height + deltaY, minHeight, desktopHeight - Y);
                break;
            case ResizeHandle.BottomRight:
                newW = Math.Clamp(Width + deltaX, minWidth, desktopWidth - X);
                newH = Math.Clamp(Height + deltaY, minHeight, desktopHeight - Y);
                break;
            case ResizeHandle.Left:
                {
                    int maxDelta = Width - minWidth;
                    int clampedDx = Math.Clamp(deltaX, -X, maxDelta);
                    newX = X + clampedDx;
                    newW = Width - clampedDx;
                }
                break;
            case ResizeHandle.Top:
                {
                    int maxDelta = Height - minHeight;
                    int clampedDy = Math.Clamp(deltaY, -Y, maxDelta);
                    newY = Y + clampedDy;
                    newH = Height - clampedDy;
                }
                break;
            case ResizeHandle.TopLeft:
                {
                    int clampedDx = Math.Clamp(deltaX, -X, Width - minWidth);
                    int clampedDy = Math.Clamp(deltaY, -Y, Height - minHeight);
                    newX = X + clampedDx;
                    newW = Width - clampedDx;
                    newY = Y + clampedDy;
                    newH = Height - clampedDy;
                }
                break;
            case ResizeHandle.TopRight:
                {
                    int clampedDy = Math.Clamp(deltaY, -Y, Height - minHeight);
                    newY = Y + clampedDy;
                    newH = Height - clampedDy;
                    newW = Math.Clamp(Width + deltaX, minWidth, desktopWidth - X);
                }
                break;
            case ResizeHandle.BottomLeft:
                {
                    int clampedDx = Math.Clamp(deltaX, -X, Width - minWidth);
                    newX = X + clampedDx;
                    newW = Width - clampedDx;
                    newH = Math.Clamp(Height + deltaY, minHeight, desktopHeight - Y);
                }
                break;
        }

        X = newX;
        Y = newY;
        Width = newW;
        Height = newH;
    }

    /// <summary>
    /// Computes active alignment snap guides relative to peer widget boxes within thresholdPx.
    /// </summary>
    public List<string> ComputeSnapGuides(IEnumerable<CanvasWidgetBox> peers, int thresholdPx = 8)
    {
        var guides = new List<string>();
        foreach (var peer in peers)
        {
            if (string.Equals(peer.WidgetId, WidgetId, StringComparison.OrdinalIgnoreCase))
                continue;

            if (Math.Abs(peer.X - X) <= thresholdPx)
                guides.Add($"AlignLeft:{peer.WidgetId}@{peer.X}");

            if (Math.Abs(peer.Y - Y) <= thresholdPx)
                guides.Add($"AlignTop:{peer.WidgetId}@{peer.Y}");

            int peerCenterX = peer.X + peer.Width / 2;
            int selfCenterX = X + Width / 2;
            if (Math.Abs(peerCenterX - selfCenterX) <= thresholdPx)
                guides.Add($"AlignCenterX:{peer.WidgetId}@{peerCenterX}");
        }
        return guides;
    }
}

