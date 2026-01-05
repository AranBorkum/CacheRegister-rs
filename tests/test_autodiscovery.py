import cache_register


def test_autodiscovery():
    assert not cache_register.get_all_registers()
    cache_register.autodiscover_registers(base_path="tests")
    cache_register.autoregister_registers(base_path="tests")
    assert cache_register.get_all_registers()

    assert "handlers" in cache_register.get_all_registers()
    handlers = cache_register.get_all_registers().get("handlers")
    assert handlers is not None
    assert "a" in handlers
