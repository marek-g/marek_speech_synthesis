import os
from typing import List
import threading
import gc
import os

class XTTS2Speech:
    def __init__(self, use_cuda, use_deepspeed, preload_on_startup) -> None:
        self.lock = threading.Lock()
        self.speaker_data = {}
        self.ref_count = 0
        self.use_cuda = use_cuda
        self.use_deepspeed = use_deepspeed
        self.preload_on_startup = preload_on_startup

    def add_reference(self):
        with self.lock:
            self.ref_count += 1
            if self.ref_count == 1 and not self.preload_on_startup:
                self.start()

    def release_reference(self):
        """Free resources if reference count drops to 0"""
        with self.lock:
            self.ref_count -= 1
            if self.ref_count == 0 and not self.preload_on_startup:
                self.stop()

    def start(self):
        print("XTTS2 engine: starting...");
        
        import torch
        from TTS.tts.configs.xtts_config import XttsConfig
        from TTS.tts.models.xtts import Xtts

        # Get device
        device = "cuda" if self.use_cuda and torch.cuda.is_available() else "cpu"
        #TODO: deepspeed compiler should improve speed on nvidia by 2x-3x
        #but I've got compilation errors with deepspeed in runtime
        use_deepspeed = True if device == "cuda" and self.use_deepspeed else False
        print("Device: {}, deepspeed: {}".format(device, use_deepspeed))

        # Init TTS
        config = XttsConfig()
        model_path = ".models/tts_models--multilingual--multi-dataset--xtts_v2"
        config.load_json(os.path.join(model_path, "config.json"))
        self.tts_model = Xtts.init_from_config(config)
        self.tts_model.load_checkpoint(config, checkpoint_dir=model_path, eval=True,
                                       use_deepspeed=use_deepspeed)
        self.tts_model.to(device)

        print("XTTS2 engine: ready");

    def stop(self):
        print("XTTS2 engine... stopping");
        
        import torch

        del self.tts_model
        gc.collect()
        if torch.cuda.is_available():
            torch.cuda.empty_cache()

        print("XTTS2 engine: stopped");
                    
    def enumerate_voices(self):
        print("XTTS2 engine: enumerate voices");
        return [{ "voice": voice,
                    "engine": "XTTS2",
                    "languages": ["en", "es", "fr", "de",
                                  "it", "pt", "pl", "tr",
                                  "ru", "nl", "cs", "ar",
                                  "zh-cn", "ja", "hu",
                                  "ko", "hi"],
                    "sample_rate": 24000 }
                  for voice in self.tts_model.speaker_manager.speaker_names]

    def say(self, text, voice, language):
        print("XTTS2 engine: say");
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
