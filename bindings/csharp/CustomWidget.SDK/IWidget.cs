// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Threading.Tasks;

namespace CustomWidget.SDK
{
    /// <summary>
    /// Represents the 13 lifecycle states of an Aether widget plugin (11 operational states + 2 canonical aliases).
    /// </summary>
    public enum WidgetState
    {
        Unloaded,
        Loading,
        Loaded,
        Mounting,
        Active,
        Mounted,
        Paused,
        Degraded,
        Unmounting,
        Unloading,
        Unmounted,
        Error,
        Quarantined
    }

    public struct TickContext
    {
        public ulong TimestampMs { get; set; }
        public float DeltaTimeMs { get; set; }
        public ulong FrameIndex { get; set; }

        public TickContext(ulong timestampMs, float deltaTimeMs, ulong frameIndex)
        {
            TimestampMs = timestampMs;
            DeltaTimeMs = deltaTimeMs;
            FrameIndex = frameIndex;
        }
    }

    public struct RectF : IEquatable<RectF>
    {
        public float X { get; set; }
        public float Y { get; set; }
        public float Width { get; set; }
        public float Height { get; set; }

        public RectF(float x, float y, float width, float height)
        {
            X = x;
            Y = y;
            Width = width;
            Height = height;
        }

        public bool Equals(RectF other) =>
            X.Equals(other.X) && Y.Equals(other.Y) && Width.Equals(other.Width) && Height.Equals(other.Height);

        public override bool Equals(object? obj) => obj is RectF other && Equals(other);
        public override int GetHashCode() => HashCode.Combine(X, Y, Width, Height);
    }

    public struct Color : IEquatable<Color>
    {
        public float R { get; set; }
        public float G { get; set; }
        public float B { get; set; }
        public float A { get; set; }

        public Color(float r, float g, float b, float a = 1.0f)
        {
            R = r;
            G = g;
            B = b;
            A = a;
        }

        public static Color Rgba(byte r, byte g, byte b, byte a = 255) =>
            new(r / 255f, g / 255f, b / 255f, a / 255f);

        public bool Equals(Color other) =>
            R.Equals(other.R) && G.Equals(other.G) && B.Equals(other.B) && A.Equals(other.A);

        public override bool Equals(object? obj) => obj is Color other && Equals(other);
        public override int GetHashCode() => HashCode.Combine(R, G, B, A);
    }

    public sealed record DrawCommandRecord(
        string Kind,
        RectF Bounds,
        Color Color,
        float CornerRadius = 0.0f,
        string? Text = null,
        string? FontFamily = null,
        float FontSize = 0.0f,
        string? ResourceId = null,
        float Opacity = 1.0f);

    public interface IRenderCanvas
    {
        void Clear(Color color);
        void DrawRect(RectF rect, Color color, float cornerRadius = 0.0f);
        void DrawText(string text, string fontFamily, float fontSize, RectF rect, Color color);
        void DrawImage(string resourceId, RectF rect, float opacity = 1.0f);
        void PushClip(RectF rect);
        void PopClip();
        void Invalidate(RectF rect);
        IReadOnlyList<DrawCommandRecord> Commands { get; }
    }

    /// <summary>
    /// Concrete retained-mode command batcher implementing <see cref="IRenderCanvas"/>.
    /// </summary>
    public sealed class BatchRenderCanvas : IRenderCanvas
    {
        private readonly List<DrawCommandRecord> _commands = new();
        private readonly List<RectF> _invalidatedRegions = new();

        public IReadOnlyList<DrawCommandRecord> Commands => _commands;
        public IReadOnlyList<RectF> InvalidatedRegions => _invalidatedRegions;

        public void Clear(Color color)
        {
            _commands.Clear();
            _commands.Add(new DrawCommandRecord("Clear", new RectF(0, 0, 0, 0), color));
        }

        public void DrawRect(RectF rect, Color color, float cornerRadius = 0.0f)
        {
            _commands.Add(new DrawCommandRecord("DrawRect", rect, color, CornerRadius: cornerRadius));
        }

        public void DrawText(string text, string fontFamily, float fontSize, RectF rect, Color color)
        {
            _commands.Add(new DrawCommandRecord(
                "DrawText",
                rect,
                color,
                Text: text,
                FontFamily: fontFamily,
                FontSize: fontSize));
        }

        public void DrawImage(string resourceId, RectF rect, float opacity = 1.0f)
        {
            _commands.Add(new DrawCommandRecord(
                "DrawImage",
                rect,
                new Color(1, 1, 1, opacity),
                ResourceId: resourceId,
                Opacity: opacity));
        }

        public void PushClip(RectF rect)
        {
            _commands.Add(new DrawCommandRecord("PushClip", rect, new Color(0, 0, 0, 0)));
        }

        public void PopClip()
        {
            _commands.Add(new DrawCommandRecord("PopClip", new RectF(0, 0, 0, 0), new Color(0, 0, 0, 0)));
        }

        public void Invalidate(RectF rect)
        {
            _invalidatedRegions.Add(rect);
        }

        public string ToJsonBatch() => JsonSerializer.Serialize(_commands);
    }

    public interface ISettingsStore
    {
        object? GetSetting(string key);
        void SetSetting(string key, object value);
    }

    /// <summary>
    /// Thread-safe in-memory implementation of <see cref="ISettingsStore"/> for C# widgets.
    /// </summary>
    public sealed class InMemorySettingsStore : ISettingsStore
    {
        private readonly Dictionary<string, object> _store = new(StringComparer.OrdinalIgnoreCase);

        public object? GetSetting(string key) =>
            _store.TryGetValue(key, out var val) ? val : null;

        public void SetSetting(string key, object value) =>
            _store[key] = value;
    }

    public interface IWidget
    {
        WidgetState State { get; }
        Task OnLoadAsync();
        Task OnMountAsync();
        void OnUpdate(in TickContext context, IRenderCanvas canvas);
        Task OnUnmountAsync();
        Task OnUnloadAsync();
        void OnEvent(string topic, string payload);
    }
}
