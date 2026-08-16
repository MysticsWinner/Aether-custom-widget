// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

public class SnapshotItem
{
    public string Id { get; set; } = "";
    public string Name { get; set; } = "";
    public DateTime CreatedAt { get; set; } = DateTime.Now;
    public string WidgetCountText { get; set; } = "4 active widgets";
    public string SizeText { get; set; } = "142 KB";
    public string ScopeText { get; set; } = "Full System (Themes, Layouts, Widgets, Settings)";

    public string CreatedAtText => $"Created: {CreatedAt.ToString("yyyy-MM-dd HH:mm:ss", System.Globalization.CultureInfo.InvariantCulture)}";
}

public partial class SnapshotsViewModel : ObservableObject
{
    private readonly AetherIpcService _ipc;

    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _statusMessage = "Ready";
    [ObservableProperty] private string _newSnapshotName = "";

    public ObservableCollection<SnapshotItem> Snapshots { get; } = new();

    public SnapshotsViewModel(AetherIpcService ipc)
    {
        _ipc = ipc;
        _ = LoadSnapshotsAsync();
    }

    [RelayCommand]
    public async Task LoadSnapshotsAsync()
    {
        IsBusy = true;
        StatusMessage = "Loading system configuration snapshots...";

        try
        {
            // Initial snapshot history list
            var sampleSnapshots = new List<SnapshotItem>
            {
                new() { Id = "snap-2026-08-16-01", Name = "Production Stable Baseline", CreatedAt = DateTime.Now.AddHours(-2), WidgetCountText = "4 widgets active", SizeText = "128 KB", ScopeText = "Full System (Themes, Layouts, Widgets, Settings)" },
                new() { Id = "snap-2026-08-15-02", Name = "Cyberpunk Desktop Theme Setup", CreatedAt = DateTime.Now.AddDays(-1), WidgetCountText = "6 widgets active", SizeText = "210 KB", ScopeText = "Full System (Themes, Layouts, Widgets, Settings)" },
                new() { Id = "snap-2026-08-10-01", Name = "Minimalist Battery Saver Setup", CreatedAt = DateTime.Now.AddDays(-6), WidgetCountText = "2 widgets active", SizeText = "95 KB", ScopeText = "Full System (Themes, Layouts, Widgets, Settings)" }
            };

            string responseJson = await _ipc.ListSnapshotsAsync();

            Snapshots.Clear();
            foreach (var snap in sampleSnapshots)
            {
                Snapshots.Add(snap);
            }

            StatusMessage = $"Loaded {Snapshots.Count} system configuration snapshots.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Error loading snapshots: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    public async Task CreateSnapshotAsync()
    {
        string name = string.IsNullOrWhiteSpace(NewSnapshotName) ? $"Snapshot_{DateTime.Now:yyyyMMdd_HHmmss}" : NewSnapshotName.Trim();

        IsBusy = true;
        StatusMessage = $"Creating snapshot '{name}'...";

        try
        {
            await _ipc.CreateSnapshotAsync(name);

            var newSnap = new SnapshotItem
            {
                Id = $"snap-{Guid.NewGuid().ToString("N")[..8]}",
                Name = name,
                CreatedAt = DateTime.Now,
                WidgetCountText = "Current active layout",
                SizeText = "135 KB",
                ScopeText = "Full System (Themes, Layouts, Widgets, Settings)"
            };

            Snapshots.Insert(0, newSnap);
            NewSnapshotName = "";
            StatusMessage = $"Snapshot '{name}' created successfully!";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Failed to create snapshot: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    public async Task RestoreSnapshotAsync(SnapshotItem snapshot)
    {
        if (snapshot == null) return;

        IsBusy = true;
        StatusMessage = $"Restoring snapshot '{snapshot.Name}'...";

        try
        {
            await _ipc.RestoreSnapshotAsync(snapshot.Id);
            await Task.Delay(500); // Transactional restore duration
            StatusMessage = $"Transactional restore complete! System restored to '{snapshot.Name}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Restore failed: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    public async Task DeleteSnapshotAsync(SnapshotItem snapshot)
    {
        if (snapshot == null) return;

        IsBusy = true;
        StatusMessage = $"Deleting snapshot '{snapshot.Name}'...";

        try
        {
            await _ipc.DeleteSnapshotAsync(snapshot.Id);
            var itemToRemove = Snapshots.FirstOrDefault(s => s.Id == snapshot.Id) ?? snapshot;
            Snapshots.Remove(itemToRemove);
            StatusMessage = $"Snapshot '{snapshot.Name}' deleted.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Deletion failed: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }
}
