import typing

# Generic type variable to preserve the type of the decorated object
T = typing.TypeVar("T")

class DuplicateRegisterEntry(Exception):
    """
    Raised when attempting to register a key that already exists
    in the specified register.
    """

    ...

class InvalidObjectInRegister(Exception):
    """
    Raised when an object being registered does not match the
    `expected_type` defined for the register.
    """

    ...

class Register(typing.Generic[T]):
    """
    A thread-safe, global registry for Python objects.
    """

    name: str

    def __init__(
        self, name: str, expected_type: type[typing.Any] | None = None
    ) -> None:
        """
        Initialize a new Register.

        :param name: Unique name for this registry namespace.
        :param expected_type: (Optional) A Python type (e.g. `int`, `MyClass`)
                              to enforce strict type checking on registered objects.
        """
        ...

    def register(self, key: str) -> typing.Callable[[T], T]:
        """
        A decorator to register an object (function, class, etc.) under a specific key.

        :param key: The unique identifier for this object within this register.
        :return: A decorator function that registers and returns the original object unmodified.
        :raises DuplicateRegisterEntry: If the key already exists.
        :raises InvalidObjectInRegister: If the object type does not match `expected_type`.
        """
        ...

    def get(self, key: str) -> T | None:
        """
        Retrieve an object from the register by key.

        :param key: The key to look up.
        :return: The registered object, or None if the key does not exist.
        """
        ...

def clear_global_register() -> None:
    """
    Clears ALL data from ALL registers globally.
    Used primarily for testing or resetting application state.
    """
    ...

def get_all_registers() -> dict[str, dict[str, typing.Any]]:
    """
    Returns a snapshot of the entire global registry state.

    :return: A dictionary where keys are register names and values
             are dictionaries of {registered_key: registered_object}.
    """
    ...

def autodiscover_registers(base_path: str = ".") -> None:
    """
    Scans the `base_path` recursively for files named `registers.py`
    (or packages named `registers`) and imports them.

    This is used to automatically load modules that define registers.

    :param base_path: The root directory to start scanning from (default: ".").
    """
    ...

def autoregister_registers(base_path: str = ".") -> None:
    """
    Scans the `base_path` recursively for files (or packages) that match
    the names of currently created registers and imports them.

    For example, if a register named "plugins" exists, this will look for
    `plugins.py` or `plugins/` and import it.

    :param base_path: The root directory to start scanning from (default: ".").
    """
    ...
