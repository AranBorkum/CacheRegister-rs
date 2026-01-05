import pytest

import cache_register

from tests.fixtures import types


class TestRegister:
    def test_register(
        self,
        _clear_global_register: None,
        primary_register: cache_register.Register,
        secondary_register: cache_register.Register,
    ) -> None:
        @primary_register.register("a")
        class A:
            pass

        assert primary_register.get("a") == A
        assert secondary_register.get("a") is None

    def test_register_many_objects_in_multiple_registers(
        self,
        _clear_global_register: None,
        primary_register: cache_register.Register,
        secondary_register: cache_register.Register,
    ) -> None:
        @primary_register.register("a")
        class A:
            pass

        @primary_register.register("b")
        class B:
            pass

        @secondary_register.register("c")
        class C:
            pass

        assert primary_register.get("a") == A
        assert primary_register.get("b") == B
        assert secondary_register.get("c") == C

        assert secondary_register.get("a") is None
        assert secondary_register.get("b") is None
        assert primary_register.get("c") is None

    def test_registering_multiple_objects_with_the_same_key(
        self, _clear_global_register: None, primary_register: cache_register.Register
    ) -> None:
        @primary_register.register("a")
        class A:
            pass

        assert primary_register.get("a") == A
        with pytest.raises(cache_register.DuplicateRegisterEntry) as e:

            @primary_register.register("a")
            class _:
                pass

        assert e.value.args[0] == "Key 'a' already exists in register 'primary'"

    def test_typed_register_only_allows_one_type(
        self,
        _clear_global_register: None,
        typed_register: cache_register.Register[types.Object],
    ) -> None:
        @typed_register.register("b")
        class B(types.Object):
            pass

        with pytest.raises(cache_register.InvalidObjectInRegister):

            @typed_register.register("c")
            class C:
                pass
