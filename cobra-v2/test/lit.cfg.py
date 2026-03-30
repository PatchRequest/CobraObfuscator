import lit.formats
import os

config.name = "cobra-v2"
config.test_format = lit.formats.ShTest(True)
config.suffixes = ['.ll']
config.test_source_root = os.path.dirname(__file__)
config.substitutions.append(('%cobra', os.path.join(
    os.path.dirname(__file__), '..', 'build', 'cobra-v2')))
