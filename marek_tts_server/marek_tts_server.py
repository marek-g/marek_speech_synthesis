import json
import socket
import socketserver
import sys
import traceback

from engines.xtts2_speech import XTTS2Speech

speech = XTTS2Speech()

class ThreadedTCPRequestHandler(socketserver.BaseRequestHandler):
    """
    The request handler class for our server.

    It is instantiated once per connection to the server, and must
    override the handle() method to implement communication to the
    client.
    """

    def setup(self):
        super().setup()
        speech.add_reference()

    def finish(self):
        speech.release_reference()
        super().finish()

    def handle(self):
        stream_reader = socket.SocketIO(self.request, "r")
        while True:
            line = stream_reader.readline().strip()
            if len(line) == 0:
                break
            try:

                command = json.loads(line)

                if command["method"] == "enumerateVoices":
                    self.enumerate_voices()

                if command["method"] == "ttsStream":
                    self.tts_stream(stream_reader, command["text"], command["voice"], command["engine"], command["language"])

            except Exception as e:

                self.request.sendall((json.dumps({
                            "resultCode": -1, "description": str(e)}) + "\n").encode())
                traceback.print_exc(file=sys.stderr)

    def enumerate_voices(self):
        voices = speech.enumerate_voices()
        result = [{ "voice": voice,
                    "engine": "XTTS2",
                    "languages": ["en", "es", "fr", "de",
                                  "it", "pt", "pl", "tr",
                                  "ru", "nl", "cs", "ar",
                                  "zh-cn", "ja", "hu",
                                  "ko", "hi"] }
                  for voice in voices]
        self.request.sendall((json.dumps(result) + "\n").encode())

    def tts_stream(self, stream_reader, text, voice, engine, language):
        if engine != "XTTS2":
            return

        chunks = speech.say(text, voice, language)
        self.request.sendall((json.dumps({
            "resultCode": 0, "description": "OK", "sample_rate": 24000}) + "\n").encode())

        zero_sent = False
        break_received = False
        for chunk in chunks:
            self.request.send(len(chunk).to_bytes(4, 'little'))
            if len(chunk) == 0:
                zero_sent = True
                break
            else:
                self.request.sendall(chunk)
                response = stream_reader.readline().strip()
                if response != "y":
                    break_reveived = True
                    break
        if not break_received and not zero_sent:
            zero = 0
            self.request.send(zero.to_bytes(4, 'little'))

class ThreadedTCPServer(socketserver.ThreadingMixIn, socketserver.TCPServer):
    pass

if __name__ == "__main__":
    HOST, PORT = "localhost", 9999

    # Create the server, binding to localhost on port 9999
    with ThreadedTCPServer((HOST, PORT), ThreadedTCPRequestHandler) as server:
        # Activate the server; this will keep running until you
        # interrupt the program with Ctrl-C
        server.serve_forever()
