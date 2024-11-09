#!/bin/sh

export CUDA_HOME=/opt/cuda
source .venv/bin/activate
python ./marek_tts_server.py
