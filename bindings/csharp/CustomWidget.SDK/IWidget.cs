using System;
using System.Threading.Tasks;

namespace CustomWidget.SDK
{
    public enum WidgetState
    {
        Unloaded,
        Loaded,
        Mounted,
        Unmounted
    }

    public struct TickContext
    {
        public ulong TimestampMs { get; set; }
        public float DeltaTimeMs { get; set; }
        public ulong FrameIndex { get; set; }
    }

    public struct RectF
    {
        public float X { get; set; }
        public float Y { get; set; }
        public float Width { get; set; }
        public float Height { get; set; }
    }

    public struct Color
    {
        public float R { get; set; }
        public float G { get; set; }
        public float B { get; set; }
        public float A { get; set; }
    }

    public interface IRenderCanvas
    {
        void Clear(Color color);
        void DrawRect(RectF rect, Color color, float cornerRadius = 0.0f);
        void DrawText(string text, string fontFamily, float fontSize, RectF rect, Color color);
        void DrawImage(string resourceId, RectF rect, float opacity = 1.0f);
        void PushClip(RectF rect);
        void PopClip();
        void Invalidate(RectF rect);
    }

    public interface ISettingsStore
    {
        object? GetSetting(string key);
        void SetSetting(string key, object value);
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
