import ctypes
import sys
from subprocess import Popen

from .game import GameTestScenario


def get_library_name() -> str:
    prefix = {'win32': ''}.get(sys.platform, 'lib')
    extension = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
    return f'{prefix}testing{extension}'


# http://jakegoulding.com/rust-ffi-omnibus/objects/
# https://jakegoulding.com/rust-ffi-omnibus/string_return/
class Any(ctypes.Structure):
    pass


Self = ctypes.POINTER(Any)


def load_testing_library(need_rebuild: bool = True) -> GameTestScenario:
    if need_rebuild:
        rebuild = Popen(
            ['cargo', 'build', '--package', 'testing'],
            cwd='../',
        )
        rebuild.wait()

    lib_name = get_library_name()
    lib = ctypes.CDLL(f"../target/debug/{lib_name}")
    lib.create.restype = Self

    return GameTestScenario(lib)
