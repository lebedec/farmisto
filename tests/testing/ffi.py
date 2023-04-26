import ctypes
from subprocess import Popen

from .game import GameTestScenario


# http://jakegoulding.com/rust-ffi-omnibus/objects/
class Any(ctypes.Structure):
    pass


Self = ctypes.POINTER(Any)


def load_testing_library(need_rebuild: bool = True) -> GameTestScenario:
    if need_rebuild:
        rebuild = Popen(
            ['cargo', 'build', '--package', 'testing', '--release'],
            cwd='../',
        )
        rebuild.wait()

    lib = ctypes.CDLL("../target/release/testing.dll")
    lib.create.restype = Self

    return GameTestScenario(lib)
