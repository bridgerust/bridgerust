import os
import sys
sys.path.insert(0, os.path.abspath('..'))

project = 'embex'
copyright = '2025, BridgeRust Team'
author = 'BridgeRust Team'
release = '0.1.0'

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.napoleon',
    'sphinx.ext.viewcode',
]

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']

html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']
