from tests.debug.debugger import Expr


class Mutex(Expr):
    @property
    def value(self) -> Expr:
        return self._concat("inner", "value")

    @property
    def locked(self) -> Expr:
        return self._concat("state", "data", "value", "value", "value", "locked")


TARGETS = Mutex("cross::blackboard::TARGETS")
