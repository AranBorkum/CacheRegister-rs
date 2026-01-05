# Cache Register ®️🦀

A high-performance, thread-safe global registry for Python, backed by Rust.

**Cache Register** allows you to decouple the definition of objects (functions, classes, strategies) from their usage. It provides a clean, decorator-based API with the speed and safety of Rust's concurrency primitives.

## ✨ Features

- 🚀 **Rust-Backed**: Global state is managed in Rust using `Arc<Mutex<HashMap>>` for thread-safe access.
- 🔒 **Type Safe**: Enforce type constraints at runtime (e.g., `Register[MyClass]`). Supports both instances and subclasses.
- 🔍 **Autodiscovery**: Automatically scan your project for `registers.py` files or plugin directories to load configurations on startup.
- 🐍 **Pythonic**: Fully typed (PEP 484) with `.pyi` stubs for excellent IDE support.

---

🚀 Quick Start

1. Create a Register

Registers are named namespaces. You can create them anywhere; they share a global Rust backend.

```python
from cache_register import Register

# Create a register named "plugins"
plugin_reg = Register("plugins")
```

2. Register Objects

Use the .register() decorator to add items.

```python
@plugin_reg.register("payment_processor")
class StripeProcessor:
    def process(self):
        print("Processing with Stripe...")

@plugin_reg.register("login_handler")
def handle_login():
    print("Logging in...")
```

3. Retrieve Objects

Access registered items from anywhere in your codebase.

```python
# Returns the class/function stored under "payment_processor"
processor_cls = plugin_reg.get("payment_processor")
processor = processor_cls()
processor.process()
```

---

🛡️ Type Safety & Generics

You can enforce strict type checking by using generic syntax. If you define a register with a type, cache_register will validate every insertion.

```python
class PaymentBase:
    pass

# This register ONLY accepts instances or subclasses of PaymentBase
payment_reg = Register[PaymentBase]("payments")

@payment_reg.register("stripe")
class Stripe(PaymentBase):  # ✅ OK: Subclass of PaymentBase
    pass

@payment_reg.register("random")
class RandomClass:          # ❌ Raises InvalidObjectInRegister
    pass
```

Note: The type checker is permissive—it allows both instances of the target type (isinstance) and subclasses (issubclass).

---

🔍 Autodiscovery

Instead of manually importing every file to trigger the decorators, you can use autodiscovery to scan your project on startup.
autodiscover_registers(base_path=".")

Scans base_path recursively for:

    Files named registers.py.

    Packages (directories with __init__.py) named registers.

```python
from cache_register import autodiscover_registers

if __name__ == "__main__":
    # Import all 'registers.py' files found in the project
    autodiscover_registers(".")
```

`autoregister_registers(base_path=".")`

A smarter scanner. If you have a register named "actions", it will look for files named actions.py or packages named actions/.

```python
from cache_register import Register, autoregister_registers

# 1. Define the register
action_reg = Register("actions")

# 2. Run scan
# This will find and import 'actions.py' if it exists in your project.
autoregister_registers(".")
```
