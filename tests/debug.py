import subprocess
import time
import os
from pathlib import Path

from pygdbmi.gdbcontroller import GdbController

ROOT = Path(__file__).resolve().parent.parent
ELF = ROOT / "build/app_embedded/riscv32imc-unknown-none-elf/release/cross"


class Debugger:
    def __enter__(self) -> "Debugger":
        gdb_path = os.getenv("GDB_PATH")
        assert gdb_path is not None, "no gdb path provided"
        self._server = subprocess.Popen(["probe-rs", "gdb", "--chip", "esp32c3", ELF])
        try:
            time.sleep(2)
            assert self._server.poll() is None, "probe-rs gdb exited"
            self._gdb = GdbController(
                [gdb_path, "--nx", "--quiet", "--interpreter=mi3", str(ELF)]
            )
            try:
                self._execute("-target-select remote :1337")
                self._execute("-exec-continue")
            except BaseException:
                self._gdb.exit()
                raise
        except BaseException:
            self._server.terminate()
            self._server.wait()
            raise
        return self

    def __exit__(self, *_) -> None:
        try:
            self._gdb.exit()
        finally:
            self._server.terminate()
            self._server.wait()

    def read(self, expr: str) -> str:
        response = self._execute("-exec-interrupt")
        print(response)
        try:
            deadline = time.monotonic() + 5
            while not any(
                r["type"] == "notify" and r["message"] == "stopped" for r in response
            ):
                if time.monotonic() > deadline:
                    raise RuntimeError("debugger failed to halt execution")
                response = self._gdb.get_gdb_response(
                    timeout_sec=1, raise_error_on_timeout=False
                )
                print(response)
            response = self._execute(f'-data-evaluate-expression "{expr}"')
        finally:
            self._execute("-exec-continue")
        return next(r["payload"] for r in response if r["type"] == "result")["value"]

    def _execute(self, command: str) -> list[dict]:
        response = self._gdb.write(command, timeout_sec=5)
        print(response)
        err = next(
            (r for r in response if r["type"] == "result" and r["message"] == "error"),
            None,
        )
        if err is not None:
            raise RuntimeError(f"{command}: {err['payload']['msg']}")
        return response


# TODO: temporary diagnosis for "failed to fill whole buffer", remove once fixed
DIAGNOSIS = {
    "find symbol": ["info address cross::blackboard::TARGETS"],
    "raw read, halted from connect": ["x/4wx &cross::blackboard::TARGETS"],
    "print, halted from connect": ["print cross::blackboard::TARGETS"],
    "print after continue and interrupt": [
        "continue &",
        "shell sleep 1",
        "interrupt",
        "shell sleep 1",
        "print cross::blackboard::TARGETS",
    ],
}


def diagnose() -> None:
    gdb_path = os.environ["GDB_PATH"]
    subprocess.run(["probe-rs", "--version"])
    for name, commands in DIAGNOSIS.items():
        print(f"===== {name} =====", flush=True)
        server = subprocess.Popen(["probe-rs", "gdb", "--chip", "esp32c3", ELF])
        try:
            time.sleep(2)
            args = [gdb_path, "--nx", "--batch", str(ELF), "-ex", "target remote :1337"]
            for command in commands:
                args += ["-ex", command]
            try:
                subprocess.run(args, timeout=30)
            except subprocess.TimeoutExpired:
                print("gdb timed out", flush=True)
        finally:
            server.terminate()
            server.wait()


if __name__ == "__main__":
    diagnose()
