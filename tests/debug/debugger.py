import subprocess
import time
import os
from pathlib import Path
from typing import Any

from pygdbmi.gdbcontroller import GdbController

ROOT = Path(__file__).resolve().parents[2]
ELF = ROOT / "build/app_embedded/riscv32imc-unknown-none-elf/release/cross"
IGNORED_CHILDREN = {"<anonymous union>"}


class Expr:
    def __init__(self, expr: "str | Expr") -> None:
        self._expr = expr.to_expr() if isinstance(expr, Expr) else expr

    def to_expr(self) -> str:
        return self._expr

    def _concat(self, *fields: str) -> "Expr":
        return Expr(".".join([self._expr, *fields]))


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
            self._execute("-target-detach")
            self._gdb.exit()
        finally:
            self._server.terminate()
            self._server.wait()

    def _value(self, var: dict) -> dict | str:
        if var["numchild"] == "0":
            return var["value"]
        command = f"-var-list-children --all-values {var['name']}"
        children = self._execute(command)["payload"]["children"]
        return {
            child["exp"]: self._value(child)
            for child in children
            if child["exp"] not in IGNORED_CHILDREN
        }

    def _execute(self, command: str) -> dict:
        response = self._gdb.write(command, timeout_sec=5)
        while not any(r["type"] == "result" for r in response):
            response += self._gdb.get_gdb_response(timeout_sec=5)
        result = next(r for r in response if r["type"] == "result")
        if result["message"] == "error":
            raise RuntimeError(f"{command}: {result['payload']['msg']}")
        return result

    def read(self, expr: Expr) -> Any:
        ex = expr.to_expr()
        self._execute(f'-data-evaluate-expression "{ex}"')
        var = self._execute(f'-var-create - * "{ex}"')["payload"]
        try:
            return self._value(var)
        finally:
            self._execute(f"-var-delete {var['name']}")

    def write(self, expr: Expr, value: str) -> None:
        self._execute(f'-data-evaluate-expression "{expr.to_expr()} = {value}"')
