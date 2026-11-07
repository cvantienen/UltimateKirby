# Removed native combo system

The old native Y Finisher/combo selector and hard-coded fighter combo libraries
are removed from the button menu. Current X/Y/D-pad and L3/R3 bindings are fighter-specific
saved input sequences, Normal button, or built-in short-hop/ledge-assist/turbo helpers. Sequences can run once or loop while the
assigned button is held; they do not guarantee a combo or bypass native recovery.

Use [Button configs](SEQUENCE_FILES.md), [editor actions](EDITOR_ACTION_CHEATSHEET.md)
and [separate assist rule lists](REACTION_RULES.md). Automatic detection/ownership
stays in code; configured actions and input delays live in `sequences.txt`.
