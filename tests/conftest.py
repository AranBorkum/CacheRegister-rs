import pytest
import cache_register
from tests.fixtures import types


@pytest.fixture
def _clear_global_register() -> None:
    cache_register.clear_global_register()


@pytest.fixture
def primary_register() -> cache_register.Register:
    return cache_register.Register("primary")


@pytest.fixture
def secondary_register() -> cache_register.Register:
    return cache_register.Register("secondary")


@pytest.fixture
def typed_register() -> cache_register.Register[types.Object]:
    return cache_register.Register[types.Object]("typed_register")
