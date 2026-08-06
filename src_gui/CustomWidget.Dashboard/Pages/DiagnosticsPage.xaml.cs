// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Diagnostics;
using System.IO;
using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.ApplicationModel.DataTransfer;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Diagnostics Page — browse log files, inspect recent logs, open folders, and execute IPC commands.
/// </summary>
public sealed partial class DiagnosticsPage : Page
{
    private readonly DiagnosticsViewModel _vm;
    private readonly ProcessManagerService _processManager;
    private readonly DispatcherTimer _refreshTimer;
    private string _logsDirectory = "";
    private string _selectedFilePath = "";

    public DiagnosticsPage()
    {
        _vm = App.Services.GetRequiredService<DiagnosticsViewModel>();
        _processManager = App.Services.GetRequiredService<ProcessManagerService>();
        this.InitializeComponent();

        // Dynamically resolve the logs directory
        _logsDirectory = Path.Combine(_processManager.WorkspaceRoot, "logs");
        if (!Directory.Exists(_logsDirectory))
        {
            try
            {
                Directory.CreateDirectory(_logsDirectory);
            }
            catch { }
        }

        RefreshLogsList();

        // 500ms timer to refresh UI stats and optionally hot-reload log file if selected
        _refreshTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(500) };
        _refreshTimer.Tick += RefreshUI;
        _refreshTimer.Start();

        this.Unloaded += (_, _) => _refreshTimer.Stop();
    }

    private void RefreshUI(object? sender, object e)
    {
        // Update error badge
        int total = _vm.WarnCount + _vm.ErrorCount;
        if (total > 0)
        {
            ErrorBadge.Visibility = Visibility.Visible;
            ErrorBadgeText.Text = total.ToString();
        }
        else
        {
            ErrorBadge.Visibility = Visibility.Collapsed;
        }

        // Auto-refresh log contents in viewer if active
        if (!string.IsNullOrEmpty(_selectedFilePath) && File.Exists(_selectedFilePath))
        {
            LoadLogFileContents(_selectedFilePath, levelFilter: LevelFilter.SelectedItem is ComboBoxItem item ? item.Content?.ToString() ?? "All" : "All");
        }
    }

    private void RefreshLogsList()
    {
        LogFilesList.Items.Clear();
        if (Directory.Exists(_logsDirectory))
        {
            try
            {
                var files = Directory.GetFiles(_logsDirectory, "*.log");
                foreach (var file in files)
                {
                    LogFilesList.Items.Add(Path.GetFileName(file));
                }

                // Auto-select first item if list is populated and nothing is selected yet
                if (LogFilesList.Items.Count > 0 && string.IsNullOrEmpty(_selectedFilePath))
                {
                    LogFilesList.SelectedIndex = 0;
                }
            }
            catch (Exception ex)
            {
                LogStatusText.Text = $"Error reading directory: {ex.Message}";
            }
        }
    }

    private void LoadLogFileContents(string filePath, string levelFilter)
    {
        try
        {
            if (!File.Exists(filePath))
            {
                LogContentBox.Text = "File does not exist.";
                return;
            }

            // Read safely using a shared read file stream to avoid locking issues
            using var fs = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
            using var reader = new StreamReader(fs);
            var lines = new List<string>();
            while (!reader.EndOfStream)
            {
                var line = reader.ReadLine();
                if (line != null)
                {
                    // Basic level filtering check
                    if (levelFilter != "All")
                    {
                        if (!line.Contains($"[{levelFilter}]") && !line.Contains($" {levelFilter} ") && !line.ToUpperInvariant().Contains(levelFilter))
                        {
                            continue;
                        }
                    }
                    lines.Add(line);
                }
            }

            // Show latest 250 lines
            if (lines.Count > 250)
            {
                lines = lines.GetRange(lines.Count - 250, 250);
            }

            string combined = string.Join(Environment.NewLine, lines);
            if (LogContentBox.Text != combined)
            {
                double currentOffset = LogScrollViewer.VerticalOffset;
                LogContentBox.Text = combined;
                
                // Keep scroll at bottom if it was previously at bottom or near it
                if (currentOffset >= LogScrollViewer.ScrollableHeight - 50)
                {
                    LogScrollViewer.ChangeView(null, LogScrollViewer.ScrollableHeight, null);
                }
            }

            LogStatusText.Text = $"Loaded {lines.Count} entries from '{Path.GetFileName(filePath)}'. Location: {_logsDirectory}";
        }
        catch (Exception ex)
        {
            LogStatusText.Text = $"Error reading log file: {ex.Message}";
        }
    }

    private void LogFilesList_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (LogFilesList.SelectedItem is string fileName)
        {
            _selectedFilePath = Path.Combine(_logsDirectory, fileName);
            SelectedLogTitle.Text = $"Recent Logs ({fileName})";
            LoadLogFileContents(_selectedFilePath, levelFilter: LevelFilter.SelectedItem is ComboBoxItem item ? item.Content?.ToString() ?? "All" : "All");
        }
        else
        {
            _selectedFilePath = "";
            SelectedLogTitle.Text = "Recent Logs (None Selected)";
            LogContentBox.Text = "";
        }
    }

    private void OpenFolderBtn_Click(object sender, RoutedEventArgs e)
    {
        try
        {
            if (Directory.Exists(_logsDirectory))
            {
                Process.Start(new ProcessStartInfo
                {
                    FileName = "explorer.exe",
                    Arguments = _logsDirectory,
                    UseShellExecute = true
                });
            }
        }
        catch (Exception ex)
        {
            LogStatusText.Text = $"Failed to open folder: {ex.Message}";
        }
    }

    private void RefreshFilesBtn_Click(object sender, RoutedEventArgs e)
    {
        RefreshLogsList();
    }

    private void CopyLogBtn_Click(object sender, RoutedEventArgs e)
    {
        if (!string.IsNullOrEmpty(LogContentBox.Text))
        {
            try
            {
                var dataPackage = new DataPackage();
                dataPackage.SetText(LogContentBox.Text);
                Clipboard.SetContent(dataPackage);
                LogStatusText.Text = "Logs successfully copied to Clipboard!";
            }
            catch (Exception ex)
            {
                LogStatusText.Text = $"Failed to copy to clipboard: {ex.Message}";
            }
        }
    }

    private void ClearLogFileBtn_Click(object sender, RoutedEventArgs e)
    {
        if (!string.IsNullOrEmpty(_selectedFilePath) && File.Exists(_selectedFilePath))
        {
            try
            {
                // Truncate the file content safely
                using (var fs = new FileStream(_selectedFilePath, FileMode.Create, FileAccess.Write, FileShare.ReadWrite))
                {
                    // Writing empty truncates
                }
                LogContentBox.Text = "";
                LogStatusText.Text = $"Cleared file '{Path.GetFileName(_selectedFilePath)}'.";
            }
            catch (Exception ex)
            {
                LogStatusText.Text = $"Failed to clear file: {ex.Message}";
            }
        }
    }

    private void SendBtn_Click(object sender, RoutedEventArgs e)
    {
        if (_vm != null)
        {
            _vm.CommandText = CommandInput.Text;
            _ = _vm.SendCommandCommand.ExecuteAsync(null);

            // Update response after a short delay
            var timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(800) };
            timer.Tick += (_, _) =>
            {
                timer.Stop();
                ResponseOutput.Text = _vm.ResponseText;
            };
            timer.Start();
        }
    }

    private void ClearBtn_Click(object sender, RoutedEventArgs e)
    {
        if (_vm != null)
        {
            _vm.ClearLogsCommand.Execute(null);
        }
    }

    private void LevelFilter_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_vm != null && LevelFilter.SelectedItem is ComboBoxItem item)
        {
            string filter = item.Content?.ToString() ?? "All";
            _vm.SelectedLogLevel = filter;
            if (!string.IsNullOrEmpty(_selectedFilePath))
            {
                LoadLogFileContents(_selectedFilePath, filter);
            }
        }
    }
}
