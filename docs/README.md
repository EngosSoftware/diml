# Delimiter-Indented Markup Language

## Overview

Parser for **Delimiter-Indented Markup Language** (DIML).

Delimiter-Indented Markup Language (DIML) is a minimal, line-based markup language
for writing hierarchical text.

Each node starts with a delimiter character, followed by an optional name and free-form content.
Indentation determines nesting, so the structure is visible at a glance.

The document itself sets the rules:
- its first character becomes the delimiter,
- and its first indentation sets the indentation unit.

There are no closing tags, no quoting and no escaping.

Free-form content runs until the next node.
