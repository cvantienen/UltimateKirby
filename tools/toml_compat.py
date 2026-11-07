"""Use the standard TOML reader, with its backport on older Python versions."""
import sys
from importlib import import_module

tomllib = import_module("tomllib" if sys.version_info >= (3, 11) else "tomli")
