use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{AudioContext, AudioBufferSourceNode, GainNode, AudioBuffer};

#[wasm_bindgen]
pub struct BrowserAudio {
    context: Option<AudioContext>,
    source: Option<AudioBufferSourceNode>,
    gain_node: Option<GainNode>,
    playing: bool,
    volume: f64,
}

#[wasm_bindgen]
impl BrowserAudio {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<BrowserAudio, JsValue> {
        let context = AudioContext::new().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let gain_node = context.create_gain().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        gain_node.connect_with_audio_node(&context.destination())
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        let audio = BrowserAudio {
            context: Some(context),
            source: None,
            gain_node: Some(gain_node),
            playing: false,
            volume: 1.0,
        };

        Ok(audio)
    }

    pub fn play(&mut self, samples: &[f32]) -> Result<(), JsValue> {
        let ctx = self.context.as_ref().ok_or_else(|| JsValue::from_str("AudioContext not initialized"))?;
        let length = samples.len().max(1) as u32;
        let buffer = ctx.create_buffer(1, length, 44100.0)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let channel_data = buffer.get_channel_data(0).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        channel_data.copy_from(samples);

        let source = ctx.create_buffer_source().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        source.set_buffer(Some(&buffer));
        source.connect_with_audio_node(&self.gain_node.as_ref().unwrap())
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        source.start().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        self.source = Some(source);
        self.playing = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), JsValue> {
        if let Some(source) = &self.source {
            source.stop().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        self.playing = false;
        Ok(())
    }

    pub fn set_volume(&mut self, volume: f64) -> Result<(), JsValue> {
        self.volume = volume;
        if let Some(gain) = &self.gain_node {
            gain.set_gain(volume).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        Ok(())
    }

    pub fn is_playing(&self) -> bool { self.playing }
    pub fn get_volume(&self) -> f64 { self.volume }
}

impl Default for BrowserAudio {
    fn default() -> Self {
        BrowserAudio::new().unwrap()
    }
}
