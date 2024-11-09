#!/bin/sh

# create virtual environment for Python
python -m venv .venv
source .venv/bin/activate

# install COQUI AI TTS
pip install coqui-tts

# install other dependencies
pip install toml
#pip install deepspeed

# create symlink to local models folder
mkdir .models
rm -f ~/.local/share/tts
ln -sr .models ~/.local/share/tts

# download XTTS2 model
tts --model_name "tts_models/multilingual/multi-dataset/xtts_v2" --speaker_idx "Luis Moray" --text "Witaj świecie! To jest test syntezy mowy w języku polskim. Podoba Ci się?" --language_idx="pl" --out_path "test.wav"
rm test.wav
