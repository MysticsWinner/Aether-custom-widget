# Aether — AI Subsystem Limitations & Roadmap

**Current Prototype State vs Target Machine Learning Integration**

---

## 1. Prototype Limitations Analysis

The `ai_engine` crate currently operates as a **Functional Skeleton**:

- **No Remote LLM API Calls**: `LayoutGenerator` uses string `contains()` matching (`if prompt.contains("4k") { (400, 600) }`).
- **Keyword Theme Color Rules**: `ThemeGenerator` maps "cyberpunk" to `#00F0FF` / `#FF0055` without dynamic tensor inference.
- **Voice Command Parsing**: `VoiceCommandProcessor` checks string sub-matches rather than running Whisper / Speech-to-Text inference models.

---

## 2. Machine Learning Upgrade Roadmap

- **Local ONNX Model Integration**: Replace string matching with ONNX Runtime (`ort` crate) for local, private ONNX model execution.
- **Whisper Speech Recognition**: Integrate native Windows Speech API or local Whisper model for real-time offline voice control.
- **LLM API Bridge**: Connect `ai_engine` to Anthropic / Gemini / OpenAI APIs via REST client for dynamic widget code generation.
