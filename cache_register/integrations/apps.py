from django.apps import AppConfig
from django.conf import settings
import cache_register


class CacheRegisterConfig(AppConfig):
    # This name must be unique in the Django project
    name = "cache_register.integrations.django"
    label = "cache_register_django"
    verbose_name = "Cache Register"

    def ready(self):
        # 1. Get the project root from Django settings
        base_path = str(getattr(settings, "BASE_DIR", "."))

        # 2. Run the Rust autodiscovery
        try:
            cache_register.autodiscover_registers(base_path)
            cache_register.autoregister_registers(base_path)
        except Exception as e:
            # We print to stderr because Django swallows some startup errors
            import sys

            print(f"Error initializing Cache Register: {e}", file=sys.stderr)
