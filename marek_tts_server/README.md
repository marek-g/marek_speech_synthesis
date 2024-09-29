# Marek TTS Server

TCP server giving access to all supported TTS engines with a unified API.

The server is written in Python. The most interesting TTS engines available at the time of writing it are written in Python. Writting the server also in Pyhon is the quickest way to integrate them. It gives an easy access from any programming language. It can also be hosted remotely.

The reason why it's a TCP server (and not REST/HTTP) is that the stateful protocol is needed to properly manage resources. When we start the server no resources are allocated. Keeping the service unused in the background is cheap. The model is loaded once the client is connected and kept in the memory as long as at least one client is connected. Once all clients are disconnected the model is removed from the memory.

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

## The protocol

### JSON Request & JSON Response format

- UTF-8 json bytes formatted in one line followed by a single '\n' byte

### Enumerate voices

Gives a list of all available voices.

- JSON Request:

``` json
{ "method": "enumerateVoices" }
```

- JSON Response:

``` json
[ { "voice": "Claribel Dervla", "engine": "XTTS2", "languages": ["en", "pl"] } ]
```

### TTS Stream

- JSON Request:

``` json
{ "method": "ttsStream", "text": "Text to speak", "voice": "Claribel Dervla", "engine": "XTTS2", "language": "pl" }
```

- JSON Response:

``` json
{ "resultCode": 0, "description": "OK", sample_rate: 24000 }
```

- Followed by chunks. Each chunk is: [number_of_samples: 4 bytes, LE] [samples: array of 16-bit LE signed ints]. After the last chunk there is sent number_of_samples: 0.

- After each chunk waits for one line response. If the response is "y\n" - the next chunk will be sent. If the response is something else, the method is stopped (no more data will be sent).
