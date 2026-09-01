# Aether — Agent Automation Workflow

**Voice Command Execution and Autonomous Scheduling**

---

## 1. Natural Language Voice Command Execution

The `VoiceCommandProcessor` inside `crates/ai_engine/src/voice.rs` parses incoming voice strings into engine actions:

```rust
pub struct VoiceCommandProcessor;

impl VoiceCommandProcessor {
    pub fn process_voice_command(cmd: &str) -> Option<VoiceAction> {
        let cmd_lower = cmd.to_lowercase();
        if cmd_lower.contains("create widget") {
            Some(VoiceAction::CreateWidget)
        } else if cmd_lower.contains("change theme") {
            Some(VoiceAction::ChangeTheme)
        } else {
            None
        }
    }
}
```

---

## 2. Workflow Automation Runner (`WorkflowAutomation`)

The `WorkflowAutomation` system executes periodic background tasks triggered by time conditions or system telemetry thresholds (e.g. automatically switching to Dark Theme at 20:00).
