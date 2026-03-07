#!/usr/bin/env python3
"""
Fix malformed BibTeX entries exported from Zotero.

Zotero sometimes exports entries without citation keys, creating invalid BibTeX like:
  @article{
    title = {...},
  }

This script adds generated keys to fix them.

Usage:
  python3 fix_bib.py /path/to/zotLib.bib
  
Or run without arguments to fix the default file:
  python3 fix_bib.py
"""

import re
import sys
import os
from pathlib import Path

def generate_key(entry_text):
    """Generate a citation key from entry content."""
    # Try to get first author
    author_match = re.search(r'author\s*=\s*\{([^}]+)\}', entry_text, re.IGNORECASE)
    if author_match:
        author = author_match.group(1).strip()
        # Get last name of first author (handle "Last, First" or "First Last")
        if ',' in author:
            last_name = author.split(',')[0].strip()
        else:
            last_name = author.split()[0] if author.split() else 'unknown'
        # Clean: keep only letters, lowercase
        author_key = re.sub(r'[^a-zA-Z]', '', last_name).lower()
    else:
        # No author - try journal
        journal_match = re.search(r'(?:short)?journal\s*=\s*\{([^}]+)\}', entry_text, re.IGNORECASE)
        if journal_match:
            # Get first word of journal
            journal = journal_match.group(1)
            words = re.findall(r'[a-zA-Z]+', journal)
            if words:
                author_key = words[0].lower()[:8]  # Use first 8 chars
            else:
                author_key = 'noauth'
        else:
            author_key = 'noauth'
    
    # Try to get year from date or year field
    date_match = re.search(r'(?:year|date)\s*=\s*\{(\d{4})', entry_text, re.IGNORECASE)
    if date_match:
        year = date_match.group(1)
    else:
        # Try to extract year from DOI or URL
        year_match = re.search(r'/(\d{4})/', entry_text)
        if year_match:
            year = year_match.group(1)
        else:
            year = 'nodate'
    
    # Try to get first significant word from title
    title_match = re.search(r'title\s*=\s*\{([^}]+)\}', entry_text, re.IGNORECASE)
    if title_match:
        title = title_match.group(1)
        # Get first word that's not all caps
        words = re.findall(r'[a-zA-Z][a-z]+[a-zA-Z]*', title)
        if words:
            title_key = words[0].lower()
        else:
            # Fallback: first alphabetic word
            words = re.findall(r'[a-zA-Z]+', title)
            title_key = words[0].lower() if words else 'paper'
    else:
        title_key = 'paper'
    
    return f"{author_key}{year}{title_key}"


def fix_bib_file(filepath):
    """Fix malformed BibTeX entries in file."""
    filepath = Path(filepath).expanduser()
    
    if not filepath.exists():
        print(f"Error: File not found: {filepath}")
        sys.exit(1)
    
    print(f"Reading {filepath}...")
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Pattern to match @type{ followed by newline and field (missing key)
    # We look for: @article{\n  title =
    pattern = r'@([a-zA-Z]+)\{\s*\n(\s+)([a-zA-Z]+)\s*='
    
    def replace_func(match):
        entry_type = match.group(1)
        indent = match.group(2)
        first_field = match.group(3)
        
        # We need to find the full entry to generate a key
        # Start from this match position
        start = match.start()
        
        # Find the entry content (everything until matching })
        brace_count = 1
        pos = match.end()
        while pos < len(content) and brace_count > 0:
            if content[pos] == '{':
                brace_count += 1
            elif content[pos] == '}':
                brace_count -= 1
            pos += 1
        
        entry_content = content[match.start():pos]
        key = generate_key(entry_content)
        
        return f'@{entry_type}{{{key},\n{indent}{first_field} ='
    
    # Count matches before fixing
    matches = list(re.finditer(pattern, content))
    num_fixes = len(matches)
    
    if num_fixes == 0:
        print("✓ No malformed entries found. File is clean.")
        return False
    
    print(f"Found {num_fixes} malformed entries (missing citation keys)")
    
    # Apply fixes
    fixed_content = re.sub(pattern, replace_func, content)
    
    # Write backup
    backup_path = filepath.with_suffix(filepath.suffix + '.bak')
    print(f"Creating backup: {backup_path}")
    with open(backup_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    # Write fixed file
    print(f"Writing fixed file: {filepath}")
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(fixed_content)
    
    print(f"✓ Fixed {num_fixes} entries")
    return True


def main():
    # Default file path
    default_file = Path.home() / "Insync/r95g10@gmail.com/Google Drive/zotLib.bib"
    
    if len(sys.argv) > 1:
        filepath = sys.argv[1]
    else:
        filepath = default_file
    
    if not Path(filepath).exists():
        print(f"Error: File not found: {filepath}")
        print(f"Usage: {sys.argv[0]} [path/to/file.bib]")
        sys.exit(1)
    
    fix_bib_file(filepath)


if __name__ == '__main__':
    main()
