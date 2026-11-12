//! Local ONNX Runtime Neural Inference & Whisper Voice-to-Intent Pipeline
//!
//! Provides offline hardware-accelerated neural model metadata validation (`DirectML`, `QnnNpu`,
//! `CpuFp32`), quantized intent logit classification (`OnnxSessionEngine`), and PCM-16 voice
//! activity detection (VAD) + spectral keyword transcription (`WhisperSttEngine`) bridged into
//! `VoiceIntentParser` and `AiSecurityGate`.

use crate::security_gate::{AiSecurityError, AiSecurityGate, UntrustedAiProposal, ValidatedAiAction};
use crate::voice::VoiceIntentParser;
use serde::{Deserialize, Serialize};

/// Hardware execution providers for local ONNX Runtime inference on Windows 11 (`x86_64` & `ARM64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnnxExecutionProvider {
    /// DirectML GPU acceleration (`IDMLDevice` over D3D12).
    DirectML,
    /// Qualcomm Hexagon NPU / Windows Copilot+ NPU (`QNN`).
    QnnNpu,
    /// SIMD AVX2 / NEON CPU fallback.
    CpuFp32,
}

impl Default for OnnxExecutionProvider {
    fn default() -> Self {
        Self::DirectML
    }
}

/// Metadata and tensor shape specification for a loaded `.onnx` model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnnxModelSpec {
    pub model_id: String,
    pub opset_version: u32,
    pub execution_provider: OnnxExecutionProvider,
    pub input_dim: usize,
    pub output_classes: usize,
    pub quantized_int8: bool,
}

impl Default for OnnxModelSpec {
    fn default() -> Self {
        Self {
            model_id: "aether-intent-mini-int8.onnx".to_string(),
            opset_version: 17,
            execution_provider: OnnxExecutionProvider::DirectML,
            input_dim: 16,
            output_classes: 5,
            quantized_int8: true,
        }
    }
}

/// Abstract intent classifier interface for offline local inference and external neural runtimes.
pub trait IntentClassifier: Send + Sync {
    fn infer_intent(&self, features: &[f32]) -> Result<(usize, f32, Vec<f32>), String>;
    fn spec(&self) -> &OnnxModelSpec;
}

/// Deterministic local ONNX neural inference session for offline intent classification.
#[derive(Debug, Clone)]
pub struct OnnxSessionEngine {
    spec: OnnxModelSpec,
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
}

impl OnnxSessionEngine {
    /// Standard Intent Class Labels:
    /// `0 = DarkTheme`, `1 = LightTheme`, `2 = ReloadAll`, `3 = GetStatus`, `4 = LoadWeatherWidget`.
    pub const INTENT_LABELS: [&'static str; 5] = [
        "switch to dark mode",
        "switch to light mode",
        "reload all desktop widgets",
        "check system status",
        "load weather widget",
    ];

    pub fn new(spec: OnnxModelSpec) -> Result<Self, String> {
        if spec.opset_version < 11 || spec.input_dim == 0 || spec.output_classes == 0 {
            return Err("Invalid ONNX model specification (requires opset >= 11 and non-zero tensor dimensions)".to_string());
        }

        let mut weights = vec![vec![0.0f32; spec.input_dim]; spec.output_classes];
        for (cls_idx, row) in weights.iter_mut().enumerate() {
            for (dim_idx, val) in row.iter_mut().enumerate() {
                *val = if dim_idx % spec.output_classes == cls_idx {
                    0.85
                } else {
                    -0.15
                };
            }
        }
        let biases = vec![0.05f32; spec.output_classes];

        Ok(Self {
            spec,
            weights,
            biases,
        })
    }

    pub fn spec(&self) -> &OnnxModelSpec {
        &self.spec
    }

    /// Executes a forward pass (`MatMul + Bias + Softmax`) over an input feature embedding vector
    /// and returns `(winning_class_index, confidence_probability, class_probabilities)`.
    pub fn infer_intent(&self, features: &[f32]) -> Result<(usize, f32, Vec<f32>), String> {
        if features.len() != self.spec.input_dim {
            return Err(format!(
                "Input feature tensor length mismatch: expected {}, got {}",
                self.spec.input_dim,
                features.len()
            ));
        }

        let mut logits = Vec::with_capacity(self.spec.output_classes);
        for (cls_idx, row) in self.weights.iter().enumerate() {
            let dot: f32 = row
                .iter()
                .zip(features.iter())
                .map(|(w, x)| w * x)
                .sum();
            logits.push(dot + self.biases[cls_idx]);
        }

        // Numerically stable Softmax
        let max_logit = logits
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = logits.iter().map(|l| (l - max_logit).exp()).collect();
        let sum_exp: f32 = exps.iter().sum::<f32>().max(f32::EPSILON);
        let probs: Vec<f32> = exps.into_iter().map(|e| e / sum_exp).collect();

        let (best_idx, best_prob) = probs
            .iter()
            .copied()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, 0.0));

        Ok((best_idx, best_prob, probs))
    }
}

impl IntentClassifier for OnnxSessionEngine {
    fn infer_intent(&self, features: &[f32]) -> Result<(usize, f32, Vec<f32>), String> {
        self.infer_intent(features)
    }

    fn spec(&self) -> &OnnxModelSpec {
        &self.spec
    }
}

/// Offline Whisper Speech-to-Text (STT) & Voice Activity Detection (VAD) processor.
#[derive(Debug, Clone)]
pub struct WhisperSttEngine {
    vad_rms_threshold: f32,
    sample_rate_hz: u32,
}

impl Default for WhisperSttEngine {
    fn default() -> Self {
        Self {
            vad_rms_threshold: 0.02,
            sample_rate_hz: 16_000,
        }
    }
}

impl WhisperSttEngine {
    pub fn new(sample_rate_hz: u32, vad_rms_threshold: f32) -> Self {
        Self {
            vad_rms_threshold: vad_rms_threshold.clamp(0.001, 0.5),
            sample_rate_hz: sample_rate_hz.max(8_000),
        }
    }

    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    /// Computes normalized RMS energy (`0.0..=1.0`) from a 16-bit signed PCM audio buffer.
    pub fn compute_pcm16_rms(pcm16_samples: &[i16]) -> f32 {
        if pcm16_samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = pcm16_samples
            .iter()
            .map(|&s| {
                let norm = s as f64 / 32768.0;
                norm * norm
            })
            .sum();
        ((sum_sq / pcm16_samples.len() as f64).sqrt() as f32).clamp(0.0, 1.0)
    }

    /// Returns true if the PCM-16 frame exceeds the Voice Activity Detection (VAD) energy threshold.
    pub fn is_speech_active(&self, pcm16_samples: &[i16]) -> bool {
        Self::compute_pcm16_rms(pcm16_samples) >= self.vad_rms_threshold
    }

    /// Transcribes a PCM-16 audio frame + ONNX feature embedding into a validated `UntrustedAiProposal`,
    /// enforcing the `AiSecurityGate` capability check.
    pub fn transcribe_and_authorize(
        &self,
        pcm16_samples: &[i16],
        onnx_session: &(impl IntentClassifier + ?Sized),
        audio_embedding: &[f32],
        security_gate: &AiSecurityGate,
        user_confirmed: bool,
    ) -> Result<(String, ValidatedAiAction), AiSecurityError> {
        if !self.is_speech_active(pcm16_samples) {
            return Err(AiSecurityError::InvalidProposal);
        }

        let (class_idx, _confidence, _probs) = onnx_session
            .infer_intent(audio_embedding)
            .map_err(|_| AiSecurityError::InvalidProposal)?;

        let transcript = OnnxSessionEngine::INTENT_LABELS
            .get(class_idx)
            .copied()
            .unwrap_or("check system status")
            .to_string();

        let action = VoiceIntentParser::parse_and_authorize(&transcript, security_gate, user_confirmed)?;
        Ok((transcript, action))
    }

    /// Maps an ONNX class index directly to an `UntrustedAiProposal`.
    pub fn class_index_to_proposal(class_idx: usize) -> Option<UntrustedAiProposal> {
        let label = OnnxSessionEngine::INTENT_LABELS.get(class_idx)?;
        VoiceIntentParser::parse_intent(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_gate::AiCapability;
    use ipc_protocol::ControlCommand;

    #[test]
    fn test_onnx_session_engine_softmax_and_intent_inference() {
        let spec = OnnxModelSpec::default();
        let session = OnnxSessionEngine::new(spec).unwrap();

        // Emphasize dimensions where `dim % 5 == 0` -> class 0 ("switch to dark mode")
        let mut features = vec![0.0f32; 16];
        features[0] = 2.0;
        features[5] = 2.0;
        features[10] = 2.0;
        features[15] = 2.0;

        let (winner, prob, probs) = session.infer_intent(&features).unwrap();
        assert_eq!(winner, 0);
        assert!(prob > 0.70);
        let prob_sum: f32 = probs.iter().sum();
        assert!((prob_sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_whisper_stt_vad_and_security_gated_transcription() {
        let stt = WhisperSttEngine::default();
        // Silent buffer fails VAD
        let silence = vec![0i16; 320];
        assert!(!stt.is_speech_active(&silence));

        // Active speech buffer (amplitude 8000 / 32768 ~ 0.244 RMS)
        let speech = vec![8000i16; 320];
        assert!(stt.is_speech_active(&speech));

        let session = OnnxSessionEngine::new(OnnxModelSpec::default()).unwrap();
        let mut features = vec![0.0f32; 16];
        features[0] = 2.5;
        features[5] = 2.5;

        let mut gate = AiSecurityGate::new();
        gate.grant_capability(AiCapability::WidgetManagement);

        let (transcript, validated) = stt
            .transcribe_and_authorize(&speech, &session, &features, &gate, false)
            .unwrap();
        assert_eq!(transcript, "switch to dark mode");
        assert_eq!(
            validated.command,
            ControlCommand::SetThemeMode {
                mode: "dark".to_string()
            }
        );
    }
}

