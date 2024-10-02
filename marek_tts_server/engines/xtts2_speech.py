import os
from typing import List
import threading
import gc
import os

class XTTS2Speech:
    def __init__(self) -> None:
        self.lock = threading.Lock()
        self.speaker_data = {}
        self.ref_count = 0

    def add_reference(self):
        with self.lock:
            self.ref_count += 1
            if self.ref_count == 1:
                self.start()

    def release_reference(self):
        """Free resources if reference count drops to 0"""
        with self.lock:
            self.ref_count -= 1
            if self.ref_count == 0:
                self.stop()

    def start(self):
        import torch
        from TTS.tts.configs.xtts_config import XttsConfig
        from TTS.tts.models.xtts import Xtts

        # Get device
        device = "cuda" if torch.cuda.is_available() else "cpu"
        # Init TTS
        config = XttsConfig()
        model_path = ".models/tts_models--multilingual--multi-dataset--xtts_v2"
        config.load_json(os.path.join(model_path, "config.json"))
        self.tts_model = Xtts.init_from_config(config)
        self.tts_model.load_checkpoint(config, checkpoint_dir=model_path, eval=True,
                                   use_deepspeed=True if device == "cuda" else False)
        self.tts_model.to(device)

    def stop(self):
        import torch

        del self.tts_model
        gc.collect()
        if torch.cuda.is_available():
            torch.cuda.empty_cache()        
                    
    def enumerate_voices(self):
        return self.tts_model.speaker_manager.speaker_names

    def say(self, text, voice, language):
        (gpt_cond_latent, speaker_embedding) = self.get_speaker_data(voice)
        chunks = self.tts_model.inference_stream(text, language, gpt_cond_latent, speaker_embedding, enable_text_splitting=True)
        for chunk in chunks:
            yield self.convert_audio_to_list_of_ints(chunk)

    def get_speaker_data(self, voice):
        if voice not in self.speaker_data:
            if voice in self.tts_model.speaker_manager.speaker_names:
                gpt_cond_latent, speaker_embedding = self.tts_model.speaker_manager.name_to_id[voice].values()
                self.speaker_data[voice] = (gpt_cond_latent, speaker_embedding)
        return self.speaker_data[voice]

    def convert_audio_to_list_of_ints(self, wav: List[float]):
        import torch
        import numpy as np
        import scipy

        # if tensor convert to numpy
        if torch.is_tensor(wav):
            wav = wav.cpu().numpy()
        if isinstance(wav, list):
            wav = np.array(wav)
        wav_norm = (wav * 32767)
        wav_norm = wav_norm.astype(np.int16)
        return wav_norm
