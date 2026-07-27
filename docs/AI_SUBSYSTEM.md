# AI Subsystem & Intelligence Specification

The **Phase 14 AI Subsystem** (`crates/ai_engine`) introduces intelligent desktop customization across 6 pillars:

---

## 🤖 The 6 Core AI Pillars

```
+---------------------------------------------------------------------------------------------------------+
|                                        THE 6 CORE AI PILLARS                                            |
+-------------------+-------------------+-------------------+-------------------+-------------------+-------------------+
| 1. Desktop        | 2. Voice          | 3. Layout         | 4. Theme          | 5. Widget         | 6. Workflow       |
|    Automation     |    Processing     |    Synthesis      |    Synthesis      |    Synthesis      |    Automation     |
| Window grouping,  | VoiceIntentParser | Flexbox bounds    | ThemeSchema       | WidgetManifest    | Trigger-Action    |
| workspace layout  | speech-to-intent  | prompt generator  | prompt generator  | prompt generator  | telemetry rules   |
+-------------------+-------------------+-------------------+-------------------+-------------------+-------------------+
```

### 1. Desktop Automation
Executes high-level desktop management commands using natural language instructions.

### 2. Voice Processing (`VoiceIntentParser`)
Translates spoken utterances into strongly-typed `ControlCommand` signals:
- `"switch to dark theme"` -> `ControlCommand::SetThemeMode { mode: "dark" }`
- `"reload all widgets"` -> `ControlCommand::ReloadAll`
- `"load weather widget"` -> `ControlCommand::LoadWidget { manifest_path: "..." }`

### 3. AI Layout Generation (`LayoutGenerator`)
Synthesizes responsive flexbox layouts tailored to monitor display resolutions.

### 4. AI Theme Generation (`ThemeGenerator`)
Generates valid `ThemeSchema` JSON files from prompt descriptions (e.g. `"cyberpunk neon"`).

### 5. AI Widget Generation (`WidgetGenerator`)
Generates declarative TOML `WidgetManifest` files from prompt specifications (e.g. `"CPU gauge overlay"`).

### 6. Workflow Automation (`WorkflowAutomationEngine`)
Evaluates trigger-condition-action rules against live telemetry feeds:
- *Rule Example*: `"When sys.cpu_usage >= 85.0% -> automatically switch to dark performance theme"`.

---

## 🛡️ Security Validation Gate

All AI-synthesized manifests, themes, and layouts pass through static schema validators (`parse_toml`, `parse_json`) and `PermissionGuard` checks before execution inside low-integrity `AppContainer` sandboxes.
