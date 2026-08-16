// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

public class DesktopProfileItem : ObservableObject
{
    public string Id { get; set; } = "";
    public string Name { get; set; } = "";
    public string Description { get; set; } = "";
    public string SpecsText { get; set; } = "";
    public int TargetFps { get; set; } = 60;
    public string Material { get; set; } = "Mica";

    private bool _isActive;
    public bool IsActive
    {
        get => _isActive;
        set => SetProperty(ref _isActive, value);
    }

    public string ButtonText => IsActive ? "✓ Active Profile" : "Activate Profile";

    public void NotifyStateChanged()
    {
        OnPropertyChanged(nameof(IsActive));
        OnPropertyChanged(nameof(ButtonText));
    }
}

public partial class ProfilesViewModel : ObservableObject
{
    private readonly AetherIpcService _ipc;

    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _statusMessage = "Ready";
    [ObservableProperty] private string _activeProfileId = "profile.coding";

    public ObservableCollection<DesktopProfileItem> Profiles { get; } = new();

    public ProfilesViewModel(AetherIpcService ipc)
    {
        _ipc = ipc;
        _ = LoadProfilesAsync();
    }

    [RelayCommand]
    public async Task LoadProfilesAsync()
    {
        IsBusy = true;
        StatusMessage = "Fetching active desktop profiles from core engine...";

        try
        {
            var defaultList = new List<DesktopProfileItem>
            {
                new() { Id = "profile.coding", Name = "Coding Profile", Description = "Optimized for VS Code, Visual Studio, and terminal development.", SpecsText = "Target FPS: 60 | Materials: Mica", TargetFps = 60, Material = "Mica", IsActive = true },
                new() { Id = "profile.gaming", Name = "Gaming Profile", Description = "Disables heavy blurs, hides non-essential widgets, locks 120+ FPS.", SpecsText = "Target FPS: 120 | Materials: Disabled", TargetFps = 120, Material = "Disabled", IsActive = false },
                new() { Id = "profile.minimal", Name = "Minimal Profile", Description = "Ultra-lightweight setup with clock and essential metrics only.", SpecsText = "Target FPS: 15 | Materials: Solid", TargetFps = 15, Material = "Solid", IsActive = false },
                new() { Id = "profile.battery", Name = "Battery Saver Profile", Description = "Reduces refresh rate to 10 FPS and halts background polling when unplugged.", SpecsText = "Target FPS: 10 | Materials: Disabled", TargetFps = 10, Material = "Disabled", IsActive = false },
                new() { Id = "profile.creative", Name = "Creative Studio Profile", Description = "Full resolution acrylic glass blur, color calibrated layout for Adobe & Photoshop.", SpecsText = "Target FPS: 60 | Materials: Acrylic", TargetFps = 60, Material = "Acrylic", IsActive = false }
            };

            // Query IPC engine for active profile
            string json = await _ipc.SendRawCommandAsync("{\"type\":\"GetActiveProfile\"}");

            Profiles.Clear();
            foreach (var p in defaultList)
            {
                p.IsActive = (p.Id == ActiveProfileId);
                Profiles.Add(p);
            }

            StatusMessage = $"Active profile: {ActiveProfileId}";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Error querying profiles: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    public async Task ActivateProfileAsync(DesktopProfileItem profile)
    {
        if (profile == null) return;

        IsBusy = true;
        StatusMessage = $"Switching active desktop profile to '{profile.Name}'...";

        try
        {
            var cmd = new { SetDesktopProfile = new { profile_id = profile.Id } };
            string json = System.Text.Json.JsonSerializer.Serialize(cmd);
            await _ipc.SendRawCommandAsync(json);

            ActiveProfileId = profile.Id;

            foreach (var p in Profiles)
            {
                p.IsActive = (p.Id == profile.Id);
                p.NotifyStateChanged();
            }

            StatusMessage = $"Active profile set to '{profile.Name}' ({profile.TargetFps} FPS)";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Failed to set profile: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }
}
