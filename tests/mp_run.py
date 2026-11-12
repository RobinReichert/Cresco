import os
import sys
import threading

from mpremote.transport_serial import SerialTransport


class Run:
    def __init__(self, port: str, script: bytes) -> None:
        self.output: list[str] = []
        self.passed = False
        self._line = b""
        self._transport = SerialTransport(port)
        self._transport.enter_raw_repl(soft_reset=True)
        self._transport.exec_raw_no_follow(script)
        self._thread = threading.Thread(target=self._follow)
        self._thread.start()

    def send(self, message: str) -> None:
        self._transport.serial.write(message.encode() + b"\n")

    def wait(self) -> bool:
        self._thread.join()
        self._transport.close()
        return self.passed

    def _follow(self) -> None:
        _, error = self._transport.follow(timeout=None, data_consumer=self._feed)
        for byte in error + b"\n" if error else b"":
            self._feed(bytes([byte]))

    def _feed(self, byte: bytes) -> None:
        if byte == b"\x04":  # end-of-output marker of the raw REPL
            return
        if byte != b"\n":
            self._line += byte
            return
        line = self._line.decode(errors="replace").rstrip("\r")
        self._line = b""
        print(line, flush=True)
        self.output.append(line)
        self.passed = self.passed or line.strip() == "PASS"
        output_file = os.environ.get("GITHUB_OUTPUT")
        if output_file and line.startswith("OUTPUT "):
            with open(output_file, "a") as f:
                f.write(line.removeprefix("OUTPUT ").strip() + "\n")


def run(script_path: str) -> Run:
    with open(script_path, "rb") as f:
        return Run(os.environ["MP_PORT"], f.read())


if __name__ == "__main__":
    sys.exit(0 if run(sys.argv[1]).wait() else 1)
