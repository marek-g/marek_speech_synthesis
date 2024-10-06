import json
import socket
import socketserver
import sys
import toml
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
        result = speech.enumerate_voices()
        self.request.sendall((json.dumps(result) + "\n").encode())

    def tts_stream(self, stream_reader, text, voice, engine, language):
        if engine != "XTTS2":
            return

        chunks = speech.say(text, voice, language)

        break_received = False
        for chunk in chunks:
            if len(chunk) == 0:
                break
            else:
                self.request.sendall((json.dumps({
                    "resultCode": 0, "sample_rate": 24000, "chunk_size": len(chunk)}) + "\n").encode())
                self.request.sendall(chunk)
                response = stream_reader.readline().strip()
                if response != "y":
                    break_received = True
                    break
        if not break_received:
            self.request.sendall((json.dumps({
                "resultCode": 0, "sample_rate": 24000, "chunk_size": 0}) + "\n").encode())

class ThreadedTCPServer(socketserver.ThreadingMixIn, socketserver.TCPServer):
    pass

if __name__ == "__main__":
    config = toml.load("config.toml")

    HOST, PORT = config["server"]["host"], config["server"]["port"]

    print("Starting TTS server at {}:{}".format(HOST, PORT))

    # Create the server, binding to localhost on port 9999
    with ThreadedTCPServer((HOST, PORT), ThreadedTCPRequestHandler) as server:
        # Activate the server; this will keep running until you
        # interrupt the program with Ctrl-C
        server.serve_forever()
