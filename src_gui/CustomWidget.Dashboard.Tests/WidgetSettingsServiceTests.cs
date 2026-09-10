// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class WidgetSettingsServiceTests
{
    private AetherIpcService _ipc = null!;
    private WidgetSettingsService _settings = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _settings = new WidgetSettingsService(_ipc);
    }

    [TestMethod]
    public void Test_Load_DefaultOptions_WhenFileDoesNotExist()
    {
        var opts = _settings.Load("test_nonexistent_widget_" + Guid.NewGuid().ToString("N"));
        Assert.IsNotNull(opts);
        Assert.AreEqual(1.0, opts.Opacity);
        Assert.AreEqual(1.0, opts.Scale);
        Assert.IsFalse(opts.Locked);
        Assert.IsTrue(opts.Enabled);
    }

    [TestMethod]
    public async Task Test_SaveAndLoad_RoundtripsSuccessfully()
    {
        string widgetId = "test_widget_" + Guid.NewGuid().ToString("N");
        var opts = new WidgetDisplayOptions
        {
            WidgetId = widgetId,
            Opacity = 0.75,
            Scale = 1.2,
            Locked = true,
            Enabled = false,
        };

        await _settings.SaveAsync(opts);
        var loaded = _settings.Load(widgetId);

        Assert.AreEqual(opts.WidgetId, loaded.WidgetId);
        Assert.AreEqual(opts.Opacity, loaded.Opacity, 0.01);
        Assert.AreEqual(opts.Scale, loaded.Scale, 0.01);
        Assert.AreEqual(opts.Locked, loaded.Locked);
        Assert.AreEqual(opts.Enabled, loaded.Enabled);
    }

    [TestMethod]
    public async Task Test_SetOpacity_ClampsValueCorrectly()
    {
        string widgetId = "test_opacity_widget_" + Guid.NewGuid().ToString("N");
        await _settings.SetOpacityAsync(widgetId, 1.5);
        var loaded = _settings.Load(widgetId);
        Assert.AreEqual(1.0, loaded.Opacity, 0.01);
    }

    [TestMethod]
    public async Task Test_ToggleLock_TogglesState()
    {
        string widgetId = "test_lock_widget_" + Guid.NewGuid().ToString("N");
        var initial = _settings.Load(widgetId);
        bool initialLocked = initial.Locked;

        await _settings.ToggleLockAsync(widgetId);
        var toggled = _settings.Load(widgetId);
        Assert.AreEqual(!initialLocked, toggled.Locked);
    }

    [TestMethod]
    public async Task Test_Reset_ClearsSettingsToDefaults()
    {
        string widgetId = "test_reset_widget_" + Guid.NewGuid().ToString("N");
        await _settings.SetOpacityAsync(widgetId, 0.3);
        await _settings.ResetAsync(widgetId);

        var resetOpts = _settings.Load(widgetId);
        Assert.AreEqual(1.0, resetOpts.Opacity, 0.01);
    }

    [TestMethod]
    public async Task Test_LoadAllSettingsAsync_ReturnsDictionary()
    {
        string widgetId = "test_loadall_" + Guid.NewGuid().ToString("N");
        var opts = new WidgetDisplayOptions
        {
            WidgetId = widgetId,
            Opacity = 0.85,
            Scale = 1.1,
            Locked = true,
            Enabled = true
        };
        await _settings.SaveSettingsAsync(widgetId, opts);

        var all = await _settings.LoadAllSettingsAsync();
        Assert.IsNotNull(all);
        Assert.IsTrue(all.ContainsKey(widgetId));
        Assert.AreEqual(0.85, all[widgetId].Opacity, 0.01);
    }

    [TestMethod]
    public async Task Test_GetSettingsAsync_ReturnsMatchingOptions()
    {
        string widgetId = "test_getasync_" + Guid.NewGuid().ToString("N");
        var opts = new WidgetDisplayOptions
        {
            WidgetId = widgetId,
            Opacity = 0.65,
            Scale = 0.9,
            Locked = false,
            Enabled = true
        };
        await _settings.SaveSettingsAsync(widgetId, opts);

        var retrieved = await _settings.GetSettingsAsync(widgetId);
        Assert.IsNotNull(retrieved);
        Assert.AreEqual(widgetId, retrieved.WidgetId);
        Assert.AreEqual(0.65, retrieved.Opacity, 0.01);
    }

    [TestMethod]
    public async Task Test_SetPositionAsync_CompletesSuccessfully()
    {
        string widgetId = "test_pos_" + Guid.NewGuid().ToString("N");
        // Should complete without throwing even if IPC is offline (best-effort)
        await _settings.SetPositionAsync(widgetId, 250, 400);
    }

    [TestMethod]
    public async Task Test_SetLockedAsync_PersistsLockState()
    {
        string widgetId = "test_setlocked_" + Guid.NewGuid().ToString("N");
        await _settings.SetLockedAsync(widgetId, true);
        var loaded = await _settings.GetSettingsAsync(widgetId);
        Assert.IsTrue(loaded.Locked);

        await _settings.SetLockedAsync(widgetId, false);
        loaded = await _settings.GetSettingsAsync(widgetId);
        Assert.IsFalse(loaded.Locked);
    }

    [TestMethod]
    public async Task Test_SetScaleAsync_ClampsMinimumScale()
    {
        string widgetId = "test_scale_" + Guid.NewGuid().ToString("N");
        await _settings.SetScaleAsync(widgetId, 1.4);
        var loaded = await _settings.GetSettingsAsync(widgetId);
        Assert.AreEqual(1.4, loaded.Scale, 0.01);

        // Clamping to >= 0.1
        await _settings.SetScaleAsync(widgetId, 0.01);
        loaded = await _settings.GetSettingsAsync(widgetId);
        Assert.AreEqual(0.1, loaded.Scale, 0.01);
    }

    [TestMethod]
    public async Task Test_SetEnabledAsync_PersistsEnabledState()
    {
        string widgetId = "test_enabled_" + Guid.NewGuid().ToString("N");
        await _settings.SetEnabledAsync(widgetId, false);
        var loaded = await _settings.GetSettingsAsync(widgetId);
        Assert.IsFalse(loaded.Enabled);

        await _settings.SetEnabledAsync(widgetId, true);
        loaded = await _settings.GetSettingsAsync(widgetId);
        Assert.IsTrue(loaded.Enabled);
    }
}
