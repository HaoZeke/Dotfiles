# Fix Zotero BibTeX Export

## Problem

Zotero sometimes exports BibTeX entries without citation keys, creating invalid format:

```bibtex
@article{
  title = {Some Title},
  ...
}
```

This should be:

```bibtex
@article{author2023title,
  title = {Some Title},
  ...
}
```

Missing keys cause `parsebib` (used by org-ref) to fail with errors like:
```
parsebib-error: 19, "Invalid character `%c'", 61
```

## Solution

Two scripts are provided:

1. **fix-zotero-bib** - Shell wrapper (use this)
2. **fix-zotero-bib.py** - Python implementation

## Usage

### Fix default Zotero file

```bash
fix-zotero-bib
```

This fixes: `~/Insync/r95g10@gmail.com/Google Drive/zotLib.bib`

### Fix specific file

```bash
fix-zotero-bib /path/to/file.bib
```

## What It Does

1. Scans for entries missing citation keys
2. Generates keys from: author + year + title (or journal if no author)
3. Creates backup as `file.bib.bak`
4. Writes fixed file in place

## Generated Key Format

- **With author**: `ohagan1978curve` (author + year + first title word)
- **Without author**: `aiaa2026weighted` (journal + year + first title word)
- **No date**: Uses `nodate` instead of year

## Workflow with Zotero

### Option 1: Manual fix after export

1. Export from Zotero
2. Run: `fix-zotero-bib`
3. Use the fixed file in Emacs/org-ref

### Option 2: Auto-run script (future)

Add to your shell config to auto-run when Zotero exports:

```bash
# In ~/.bashrc or similar
alias zotero-fix='fix-zotero-bib'
```

## Long-term Solution

Install **Better BibTeX** plugin for Zotero:
- https://retorque.re/zotero-better-bibtex/installation/
- Automatically generates citation keys
- Prevents this issue permanently

## Files

- Script: `~/.local/bin/fix-zotero-bib`
- Python: `~/.local/bin/fix-zotero-bib.py`
- This doc: `~/.local/bin/FIX-ZOTERO-BIB.md`

## Examples

```bash
# Fix default file
fix-zotero-bib

# Fix specific file
fix-zotero-bib ~/Documents/library.bib

# Check what was changed
diff -u file.bib.bak file.bib | head -50
```
