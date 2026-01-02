from .registers import handlers


@handlers.register("a")
class SomeHandler:
    pass
