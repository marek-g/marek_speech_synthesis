# Marek TTS Server

HTTP server giving access to all supported TTS engines with a unified API.

The server is written in Python. The most interesting TTS engines available at the time of writing it are written in Python. Writting the server also in Pyhon is the quickest way to integrate them. As the API is made in the form of HTTP server it gives an easy access from any programming language. It can also be hosted remotely.

## Supported TTS Engines

### COQUI AI TTS

A deep learning toolkit for Text-to-Speech. MPL-2.0 license.

Forked version: https://github.com/idiap/coqui-ai-TTS (pip package: `coqui-tts`)
Original link: https://github.com/coqui-ai/TTS (pip package: `TTS`)

The reason to use fork is, the original project stopped being maintained at January of 2024 and doesn't work with `Python 3.12` (as of September of 2024).

Supported models:
- `XTTS v2.0` - multi-linugual (17 languages), voice cloning, 24 kHz, CPML license, 1.7 GB (https://huggingface.co/coqui/XTTS-v2)

## Installation

### Linux

Run the script to setup python environment with TTS engines (about 6.2GB) and models (about 1.7GB). Tested with Python 3.12.5.

``` shell
./setup.sh
```
